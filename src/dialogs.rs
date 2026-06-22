use std::env;
use std::fmt;
use std::path::Path;
use std::error::Error;
use std::process::Command;

// Supported image extensions for file filters
pub const IMAGE_GLOB: &str = "*.jpg *.jpeg *.png *.tiff *.bmp *.JPG *.JPEG *.PNG *.TIFF *.BMP";

// ---------------------------------------------------------------------------
// DialogError
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DialogError {
    pub message: String,
}

impl DialogError {
    pub fn new(msg: &str) -> Self {
        DialogError {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for DialogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for DialogError {}

// ---------------------------------------------------------------------------
// FileDialogResult
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum FileDialogResult {
    /// Single mode: selected file path as String, or None if cancelled
    Single(Option<String>),
    /// Multi mode: list of selected file paths (empty Vec if cancelled)
    Multiple(Vec<String>),
}

// ---------------------------------------------------------------------------
// which — locates a binary on PATH (analogous to shutil.which)
// ---------------------------------------------------------------------------

fn which(cmd: &str) -> Option<String> {
    let path_env = env::var("PATH").ok()?;
    let separator = if cfg!(windows) { ';' } else { ':' };

    for dir in path_env.split(separator) {
        if dir.is_empty() {
            continue;
        }
        let exe_name = if cfg!(windows) {
            format!("{}.exe", cmd)
        } else {
            cmd.to_string()
        };
        let candidate = Path::new(dir).join(&exe_name);
        if candidate.is_file() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = std::fs::metadata(&candidate) {
                    if meta.permissions().mode() & 0o111 != 0 {
                        return Some(candidate.to_string_lossy().to_string());
                    }
                }
            }
            #[cfg(not(unix))]
            {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// open_file_dialog
// ---------------------------------------------------------------------------

/// Open a native file selection dialog.
///
/// Automatically detects zenity or kdialog on PATH and uses whichever
/// is available. Returns parsed file path(s) or None/empty list on
/// cancellation or error.
///
/// Args:
///     multiple: If True, allow selecting multiple files.
///     title: Optional custom dialog title.
///
/// Returns:
///     - single mode: FileDialogResult::Single(Option<String>)
///     - multi mode: FileDialogResult::Multiple(Vec<String>)
///
/// Raises:
///     DialogError: If no supported dialog backend is found on PATH.
pub fn open_file_dialog(
    multiple: bool,
    title: Option<String>,
) -> Result<FileDialogResult, DialogError> {
    let title = title.unwrap_or_else(|| {
        if multiple {
            "Select Images".to_string()
        } else {
            "Select Image".to_string()
        }
    });

    if which("zenity").is_some() {
        run_zenity(multiple, &title)
    } else if which("kdialog").is_some() {
        run_kdialog(multiple, &title)
    } else {
        Err(DialogError::new(
            "No file dialog backend found. Install zenity or kdialog.",
        ))
    }
}

/// Execute zenity file selection dialog.
fn run_zenity(multiple: bool, title: &str) -> Result<FileDialogResult, DialogError> {
    let mut cmd = Command::new("zenity");
    cmd.arg("--file-selection")
        .arg("--title")
        .arg(title)
        .arg("--file-filter")
        .arg(format!("Image Files | {}", IMAGE_GLOB))
        .arg("--file-filter")
        .arg("All Files | *");

    if multiple {
        cmd.args(["--multiple", "--separator", "\n"]);
    }

    let output = cmd
        .output()
        .map_err(|e| DialogError::new(&format!("Failed to execute zenity: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout_trimmed = stdout.trim();

    if output.status.code().unwrap_or(-1) != 0 || stdout_trimmed.is_empty() {
        return Ok(if multiple {
            FileDialogResult::Multiple(Vec::new())
        } else {
            FileDialogResult::Single(None)
        });
    }

    let paths: Vec<String> = stdout_trimmed
        .split('\n')
        .filter(|p| !p.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(if multiple {
        FileDialogResult::Multiple(paths)
    } else {
        FileDialogResult::Single(paths.into_iter().next())
    })
}

/// Execute kdialog file selection dialog.
fn run_kdialog(multiple: bool, title: &str) -> Result<FileDialogResult, DialogError> {
    let home = env::var("HOME").unwrap_or_else(|_| "~".to_string());

    let mut cmd = Command::new("kdialog");
    cmd.arg("--getopenfilename")
        .arg(&home)
        .arg(IMAGE_GLOB);

    if multiple {
        cmd.args(["--multiple", "--separate-output"]);
    }

    cmd.arg("--title").arg(title);

    let output = cmd
        .output()
        .map_err(|e| DialogError::new(&format!("Failed to execute kdialog: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout_trimmed = stdout.trim();

    if output.status.code().unwrap_or(-1) != 0 || stdout_trimmed.is_empty() {
        return Ok(if multiple {
            FileDialogResult::Multiple(Vec::new())
        } else {
            FileDialogResult::Single(None)
        });
    }

    let paths: Vec<String> = stdout_trimmed
        .split('\n')
        .filter(|p| !p.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(if multiple {
        FileDialogResult::Multiple(paths)
    } else {
        FileDialogResult::Single(paths.into_iter().next())
    })
}
