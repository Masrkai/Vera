use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use indexmap::IndexMap;

// ---------------------------------------------------------------------------
// Exiv2NotFoundError
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Exiv2NotFoundError;

impl fmt::Display for Exiv2NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "exiv2 binary not found on PATH. Install it \
            (e.g. `apt install exiv2` or `brew install exiv2`) \
            and ensure it's on PATH."
        )
    }
}

impl Error for Exiv2NotFoundError {}

// ---------------------------------------------------------------------------
// ExifCount — represents a tag "count" that may be an integer or a string
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum ExifCount {
    Int(usize),
    Str(String),
}

impl fmt::Display for ExifCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExifCount::Int(n) => write!(f, "{}", n),
            ExifCount::Str(s) => write!(f, "{}", s),
        }
    }
}

// ---------------------------------------------------------------------------
// ExifTag — a single EXIF tag as returned by extract()
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct ExifTag {
    pub tag_type: String,
    pub count: ExifCount,
    pub value: String,
    pub translated: String,
}

// ---------------------------------------------------------------------------
// ExifValue — input value for the edit() method
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum ExifValue {
    /// A plain value: "NIKON"
    Plain(String),
    /// A (type, value) pair for explicit typing:
    /// ("Rational", "43/1 28/1 2814/1000")
    Typed(String, String),
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct EditResult {
    pub status: String, // "success" | "partial"
    pub applied: Vec<String>,
    pub failed: Vec<String>,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct DeleteResult {
    pub status: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// CompletedProcess — analogous to subprocess.CompletedProcess
// ---------------------------------------------------------------------------

struct CompletedProcess {
    returncode: i32,
    stdout: String,
    stderr: String,
}

// ---------------------------------------------------------------------------
// ParsedTag — intermediate struct used by parse_print_output
// ---------------------------------------------------------------------------

struct ParsedTag {
    tag_type: String,
    count: ExifCount,
    value: String,
}

// ---------------------------------------------------------------------------
// find_exiv2 — locates the exiv2 binary on PATH
// ---------------------------------------------------------------------------

fn find_exiv2() -> Result<String, Exiv2NotFoundError> {
    let path_env = std::env::var("PATH").unwrap_or_default();
    let separator = if cfg!(windows) { ';' } else { ':' };

    for dir in path_env.split(separator) {
        if dir.is_empty() {
            continue;
        }
        let exe_name = if cfg!(windows) { "exiv2.exe" } else { "exiv2" };
        let candidate = Path::new(dir).join(exe_name);
        if candidate.is_file() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = fs::metadata(&candidate) {
                    if meta.permissions().mode() & 0o111 != 0 {
                        return Ok(candidate.to_string_lossy().to_string());
                    }
                }
            }
            #[cfg(not(unix))]
            {
                return Ok(candidate.to_string_lossy().to_string());
            }
        }
    }

    Err(Exiv2NotFoundError)
}

// ---------------------------------------------------------------------------
// split_whitespace_max — analogous to Python's str.split(None, maxsplit)
// ---------------------------------------------------------------------------

fn split_whitespace_max(s: &str, max_splits: usize) -> Vec<&str> {
    let mut result: Vec<&str> = Vec::new();
    let mut rest = s.trim_start_matches(|c: char| c.is_whitespace());

    for _ in 0..max_splits {
        if rest.is_empty() {
            return result;
        }
        if let Some(pos) = rest.find(|c: char| c.is_whitespace()) {
            result.push(&rest[..pos]);
            rest = rest[pos..]
                .trim_start_matches(|c: char| c.is_whitespace());
        } else {
            result.push(rest);
            return result;
        }
    }

    // Push the remainder (trailing whitespace preserved; caller will trim)
    if !rest.is_empty() {
        result.push(rest);
    }

    result
}

// ---------------------------------------------------------------------------
// extract_warned_keys — parses "Warning: <key>: <reason>" lines from stderr
// ---------------------------------------------------------------------------

fn extract_warned_keys(stderr: &str) -> HashSet<String> {
    let mut keys = HashSet::new();
    for line in stderr.lines() {
        if let Some(rest) = line.strip_prefix("Warning: ") {
            // Get the first whitespace-delimited token
            let token_end = rest
                .find(|c: char| c.is_whitespace())
                .unwrap_or(rest.len());
            let token = &rest[..token_end];
            // Find the last colon in the token (greedy match like \S+:)
            if let Some(colon_pos) = token.rfind(':') {
                if colon_pos > 0 {
                    keys.insert(token[..colon_pos].to_string());
                }
            }
        }
    }
    keys
}

// ---------------------------------------------------------------------------
// EXIFProcessor
// ---------------------------------------------------------------------------

pub struct EXIFProcessor {
    pub image_path: String,
    exiv2_bin: String,
}

impl EXIFProcessor {
    /// Creates a new EXIFProcessor. Returns Err(Exiv2NotFoundError)
    /// if the exiv2 binary cannot be located on PATH.
    pub fn new(image_path: String) -> Result<Self, Exiv2NotFoundError> {
        let exiv2_bin = find_exiv2()?;
        Ok(Self {
            image_path,
            exiv2_bin,
        })
    }

    /// Runs exiv2 with the given args; returns Err if check=true and
    /// exiv2 exits with a non-zero status.
    fn run(&self, args: &[&str], check: bool) -> Result<CompletedProcess, Box<dyn Error>> {
        let output = Command::new(&self.exiv2_bin).args(args).output()?;

        let returncode = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if check && returncode != 0 {
            return Err(format!(
                "exiv2 exited with code {}: {}",
                returncode,
                stderr.trim()
            )
            .into());
        }

        Ok(CompletedProcess {
            returncode,
            stdout,
            stderr,
        })
    }

    /// Parses exiv2 -pe / -pt style output into HashMap<String, ParsedTag>.
    ///
    /// Each line looks like:
    ///     Exif.GPSInfo.GPSLatitude    Rational    3    43/1 28/1 2814/1000
    ///
    /// Keys and types are always whitespace-free tokens, so splitting with
    /// maxsplit=3 is safe regardless of column padding width, even though
    /// the trailing value itself may contain spaces.
    fn parse_print_output(stdout: &str) -> IndexMap<String, ParsedTag> {
        let mut parsed: IndexMap<String, ParsedTag> = IndexMap::new();
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let parts = split_whitespace_max(line, 3);
            if parts.len() < 4 {
                continue;
            }
            let key = parts[0].to_string();
            let tag_type = parts[1].to_string();
            let count_str = parts[2];
            let count = if !count_str.is_empty()
                && count_str.chars().all(|c| c.is_ascii_digit())
            {
                ExifCount::Int(count_str.parse().unwrap_or(0))
            } else {
                ExifCount::Str(count_str.to_string())
            };
            let value = parts[3].trim().to_string();
            parsed.insert(
                key,
                ParsedTag {
                    tag_type,
                    count,
                    value,
                },
            );
        }
        parsed
    }

    /// Extracts EXIF data via exiv2, keyed by fully-qualified exiv2 keys.
    ///
    /// Returns:
    ///   {
    ///     "Exif.Image.Make": {
    ///         type: "Ascii", count: Int(6),
    ///         value: "NIKON", translated: "NIKON"
    ///     },
    ///     "Exif.GPSInfo.GPSLatitude": {
    ///         type: "Rational", count: Int(3),
    ///         value: "43/1 28/1 2814/1000",
    ///         translated: "43 deg 28' 2.81\""
    ///     },
    ///     ...
    ///   }
    pub fn extract(&self) -> Result<IndexMap<String, ExifTag>, Box<dyn Error>> {
        self.extract_inner().map_err(|e| {
            if e.downcast_ref::<Exiv2NotFoundError>().is_some() {
                e
            } else {
                format!("Failed to extract EXIF data: {}", e).into()
            }
        })
    }

    fn extract_inner(&self) -> Result<IndexMap<String, ExifTag>, Box<dyn Error>> {
        let raw_result = self.run(&["-pe", self.image_path.as_str()], true)?;
        let translated_result = self.run(&["-pt", self.image_path.as_str()], true)?;

        let raw_tags = Self::parse_print_output(&raw_result.stdout);
        let translated_tags = Self::parse_print_output(&translated_result.stdout);

        let mut exif_data: IndexMap<String, ExifTag> = IndexMap::new();
        for (key, info) in &raw_tags {
            let translated_value = translated_tags.get(key).map(|t| t.value.clone());
            let translated = translated_value.unwrap_or_else(|| info.value.clone());
            exif_data.insert(
                key.clone(),
                ExifTag {
                    tag_type: info.tag_type.clone(),
                    count: info.count.clone(),
                    value: info.value.clone(),
                    translated,
                },
            );
        }

        Ok(exif_data)
    }

    /// Sets EXIF tags via an exiv2 modify command file (one subprocess call
    /// for the whole batch, rather than one per tag).
    ///
    /// `exif_data` maps fully-qualified exiv2 keys to either:
    ///   - ExifValue::Plain(value):
    ///       {"Exif.Image.Make": ExifValue::Plain("NIKON".into())}
    ///   - ExifValue::Typed(type, value) for explicit typing (needed when
    ///     adding a brand-new tag that doesn't already exist in the file):
    ///       {"Exif.GPSInfo.GPSLatitude":
    ///           ExifValue::Typed("Rational".into(), "43/1 28/1 2814/1000".into())}
    ///
    /// exiv2 applies every valid command in the batch even when one command
    /// fails (e.g. a malformed Rational value) — it just exits non-zero and
    /// prints "Warning: <key>: <reason>" to stderr for the ones it skipped.
    /// So a non-zero exit here does NOT mean nothing was saved; this method
    /// parses stderr to report exactly which keys succeeded and which were
    /// skipped, only raising outright if *nothing* in the batch applied.
    ///
    /// Returns:
    ///   EditResult { status, applied, failed, message }
    pub fn edit(
        &self,
        exif_data: &HashMap<String, ExifValue>,
        preserve_original: bool,
    ) -> Result<EditResult, Box<dyn Error>> {
        self.edit_inner(exif_data, preserve_original).map_err(|e| {
            if e.downcast_ref::<Exiv2NotFoundError>().is_some() {
                e
            } else {
                format!("Failed to edit EXIF data: {}", e).into()
            }
        })
    }

    fn edit_inner(
        &self,
        exif_data: &HashMap<String, ExifValue>,
        preserve_original: bool,
    ) -> Result<EditResult, Box<dyn Error>> {
        if preserve_original {
            let backup_path = format!("{}.backup", self.image_path);
            fs::copy(&self.image_path, &backup_path)?;
        }

        let mut keys: Vec<String> = Vec::new();
        let mut commands: Vec<String> = Vec::new();
        for (key, value) in exif_data {
            keys.push(key.clone());
            match value {
                ExifValue::Typed(tag_type, tag_value) => {
                    commands.push(format!("set {} {} {}", key, tag_type, tag_value));
                }
                ExifValue::Plain(val) => {
                    commands.push(format!("set {} {}", key, val));
                }
            }
        }

        // Create temp command file
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let cmd_file_path = std::env::temp_dir().join(format!(
            "exiv2_cmds_{}_{}.cmds",
            std::process::id(),
            nanos
        ));

        {
            let mut cmd_file = fs::File::create(&cmd_file_path)?;
            writeln!(cmd_file, "{}", commands.join("\n"))?;
        }

        let cmd_file_path_str = cmd_file_path
            .to_str()
            .ok_or_else(|| -> Box<dyn Error> {
                "Temp file path is not valid UTF-8".into()
            })?;

        // Run exiv2 with check=false (we handle errors ourselves)
        let run_result = self.run(
            &["-m", cmd_file_path_str, self.image_path.as_str()],
            false,
        );

        // Cleanup temp file (equivalent to Python's finally block)
        let _ = fs::remove_file(&cmd_file_path);

        let result = run_result?;

        // exiv2 warns per-key as "Warning: <key>: <reason>" for any
        // command it couldn't apply; unrelated structural warnings
        // (e.g. "Warning: Directory GPSInfo has an unexpected ...")
        // don't have a colon right after the first token, so they don't
        // match and won't be mistaken for one of our keys.
        let warned_keys = extract_warned_keys(&result.stderr);
        let failed: Vec<String> = keys
            .iter()
            .filter(|k| warned_keys.contains(*k))
            .cloned()
            .collect();
        let applied: Vec<String> = keys
            .iter()
            .filter(|k| !warned_keys.contains(*k))
            .cloned()
            .collect();

        if result.returncode != 0 && applied.is_empty() {
            let stderr_trimmed = result.stderr.trim();
            let msg = if stderr_trimmed.is_empty() {
                format!("exit code {}", result.returncode)
            } else {
                stderr_trimmed.to_string()
            };
            return Err(msg.into());
        }

        let message = if failed.is_empty() {
            format!("Updated {} tag(s)", applied.len())
        } else {
            format!(
                "Updated {} tag(s), {} failed",
                applied.len(),
                failed.len()
            )
        };

        Ok(EditResult {
            status: if failed.is_empty() {
                "success".to_string()
            } else {
                "partial".to_string()
            },
            applied,
            failed,
            message,
        })
    }

    /// Deletes all EXIF data from the image via the exiv2 `rm` action.
    pub fn delete(&self, preserve_original: bool) -> Result<DeleteResult, Box<dyn Error>> {
        self.delete_inner(preserve_original).map_err(|e| {
            if e.downcast_ref::<Exiv2NotFoundError>().is_some() {
                e
            } else {
                format!("Failed to delete EXIF data: {}", e).into()
            }
        })
    }

    fn delete_inner(&self, preserve_original: bool) -> Result<DeleteResult, Box<dyn Error>> {
        if preserve_original {
            let backup_path = format!("{}.backup", self.image_path);
            fs::copy(&self.image_path, &backup_path)?;
        }

        self.run(&["-d", "a", "rm", self.image_path.as_str()], true)?;

        Ok(DeleteResult {
            status: "success".to_string(),
            message: "EXIF data deleted successfully".to_string(),
        })
    }
}
