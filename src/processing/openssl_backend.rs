use std::error::Error;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct OpensslNotFoundError;

impl fmt::Display for OpensslNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "openssl binary not found on PATH. Install it \
            (e.g. `apt install openssl` or `brew install openssl`) \
            and ensure it's on PATH."
        )
    }
}

impl Error for OpensslNotFoundError {}

pub struct OpensslProcessor;

impl OpensslProcessor {
    pub fn find_openssl() -> Result<String, OpensslNotFoundError> {
        let path_env = std::env::var("PATH").unwrap_or_default();
        let separator = if cfg!(windows) { ';' } else { ':' };

        for dir in path_env.split(separator) {
            if dir.is_empty() {
                continue;
            }
            let exe_name = if cfg!(windows) { "openssl.exe" } else { "openssl" };
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

        Err(OpensslNotFoundError)
    }

    /// Encrypt plaintext using AES-256-CBC with PBKDF2, salt, and base64 encoding (-a).
    pub fn encrypt(plaintext: &str, passphrase: &str) -> Result<String, Box<dyn Error>> {
        let openssl_path = Self::find_openssl()?;

        let mut child = Command::new(openssl_path)
            .args(&[
                "enc",
                "-aes-256-cbc",
                "-salt",
                "-pbkdf2",
                "-a",
                "-pass",
                &format!("pass:{}", passphrase),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(plaintext.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            let ciphertext = String::from_utf8(output.stdout)?;
            Ok(ciphertext.trim().to_string())
        } else {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            Err(format!("OpenSSL encryption failed: {}", err_msg).into())
        }
    }

    /// Decrypt base64-encoded AES-256-CBC ciphertext using passphrase.
    pub fn decrypt(ciphertext: &str, passphrase: &str) -> Result<String, Box<dyn Error>> {
        let openssl_path = Self::find_openssl()?;

        let mut child = Command::new(openssl_path)
            .args(&[
                "enc",
                "-d",
                "-aes-256-cbc",
                "-pbkdf2",
                "-a",
                "-pass",
                &format!("pass:{}", passphrase),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(ciphertext.as_bytes())?;
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            let plaintext = String::from_utf8(output.stdout)?;
            Ok(plaintext)
        } else {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            Err(format!("OpenSSL decryption failed (incorrect password or corrupted data): {}", err_msg).into())
        }
    }
}
