//! Provides an easy way to run Tailwind and use its output for `paxhtml` applications.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Custom error type for paxhtml_tailwind operations
///
/// This enum provides specific error variants for different failure scenarios,
/// making error handling more predictable and allowing consumers to handle
/// specific error cases appropriately.
#[derive(Debug)]
pub enum TailwindError {
    /// IO errors (file operations, process execution, etc.)
    Io(io::Error),
    /// UTF-8 conversion errors
    Utf8(std::string::FromUtf8Error),
    /// Process execution failed
    ProcessExecution {
        stdout: String,
        stderr: String,
        exit_code: Option<i32>,
    },
    /// Failed to read tailwind version
    VersionRead,
    /// Unsupported platform
    UnsupportedPlatform,
    /// Failed to download tailwind executable
    DownloadFailed { error: std::io::Error },
    /// Failed to execute tailwind command
    ExecutionFailed(String),
}
impl fmt::Display for TailwindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TailwindError::Io(err) => write!(f, "IO error: {err}"),
            TailwindError::Utf8(err) => write!(f, "UTF-8 conversion error: {err}"),
            TailwindError::ProcessExecution {
                stdout,
                stderr,
                exit_code,
            } => {
                write!(
                    f,
                    "Process execution failed (exit code: {exit_code:?}): stdout: {stdout}, stderr: {stderr}"
                )
            }
            TailwindError::VersionRead => write!(f, "Failed to read tailwind version"),
            TailwindError::UnsupportedPlatform => write!(f, "Unsupported platform"),
            TailwindError::DownloadFailed { error } => write!(f, "Download failed: {error}"),
            TailwindError::ExecutionFailed(msg) => write!(f, "Execution failed: {msg}"),
        }
    }
}
impl std::error::Error for TailwindError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TailwindError::Io(err) => Some(err),
            TailwindError::Utf8(err) => Some(err),
            _ => None,
        }
    }
}
impl From<io::Error> for TailwindError {
    fn from(err: io::Error) -> Self {
        TailwindError::Io(err)
    }
}
impl From<std::string::FromUtf8Error> for TailwindError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        TailwindError::Utf8(err)
    }
}

/// Result type alias for paxhtml_tailwind operations
pub type Result<T> = std::result::Result<T, TailwindError>;

/// The version of Tailwind CSS that was last tested with this crate.
pub const RECOMMENDED_VERSION: &str = "4.1.11";

pub enum Tailwind {
    Local(PathBuf),
    Global,
}
impl Tailwind {
    /// Download a version of Tailwind CSS's CLI, or reuse an existing version on disk
    pub fn download(version: &str, fast: bool) -> Result<Tailwind> {
        let output_path = if cfg!(target_os = "windows") {
            PathBuf::from("tailwind.exe")
        } else {
            PathBuf::from("tailwind")
        };

        // Check if executable exists and has correct version
        if output_path.exists() {
            if fast {
                // We skip the version check if fast mode is enabled
                return Ok(Tailwind::Local(output_path));
            }

            let output = Command::new(output_path.canonicalize()?).output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let Some(first_line) = stdout.lines().next() else {
                return Err(TailwindError::VersionRead);
            };
            if first_line.contains(&format!("v{version}")) {
                return Ok(Tailwind::Local(output_path));
            }
        }

        let url = {
            let executable_name = if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
                "tailwindcss-windows-x64.exe"
            } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
                "tailwindcss-macos-arm64"
            } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
                "tailwindcss-macos-x64"
            } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
                "tailwindcss-linux-arm64"
            } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
                "tailwindcss-linux-x64"
            } else {
                return Err(TailwindError::UnsupportedPlatform);
            };
            format!(
                "https://github.com/tailwindlabs/tailwindcss/releases/download/v{version}/{executable_name}"
            )
        };

        // Download using OS-specific commands
        if cfg!(target_os = "windows") {
            // Use PowerShell's Invoke-WebRequest (aliased as curl)
            let command = format!(
                "$ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -Uri '{url}' -OutFile '{}'",
                output_path.display()
            );
            Command::new("powershell")
                .args(["-Command", &command])
                .status()
                .map_err(|e| TailwindError::DownloadFailed { error: e })?;
        } else {
            // Use curl for Unix systems (Linux/macOS)
            Command::new("curl")
                .args(["-L", "-o", output_path.to_str().unwrap(), &url])
                .status()
                .map_err(|e| TailwindError::DownloadFailed { error: e })?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&output_path, std::fs::Permissions::from_mode(0o755))?;
        }

        Ok(Tailwind::Local(output_path))
    }

    /// Use the globally-installed `tailwindcss` executable. Does not check if it exists.
    pub fn global() -> Tailwind {
        Tailwind::Global
    }

    /// Takes a path to a CSS file that imports Tailwind style, and outputs the generated CSS.
    pub fn generate_from_file(&self, input_path: &Path) -> Result<String> {
        self.run_with_args(&["--input", input_path.to_str().unwrap(), "--output", "-"])
    }

    /// Run the Tailwind executable with the given arguments.
    pub fn run_with_args(&self, args: &[&str]) -> Result<String> {
        let mut command = match self {
            Tailwind::Local(path) => {
                let mut cmd = Command::new(path.canonicalize()?);
                cmd.args(args);
                cmd
            }
            Tailwind::Global => {
                let (shell, flag) = if cfg!(target_os = "windows") {
                    ("cmd", "/C")
                } else {
                    ("sh", "-c")
                };

                let mut cmd = Command::new(shell);
                cmd.args([flag, &format!("tailwindcss {}", args.join(" "))]);
                cmd
            }
        };

        let output = command.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        if !output.status.success() {
            return Err(TailwindError::ProcessExecution {
                stdout: stdout.to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code(),
            });
        }

        Ok(stdout.to_string())
    }
}
