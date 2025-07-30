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

/// Downloads the requested version of Tailwind, if required, and runs it.
///
/// # Examples
///
/// ```no_run
/// use paxhtml_tailwind::download_and_run;
///
/// match download_and_run(paxhtml_tailwind::RECOMMENDED_VERSION, false, "src/styles/tailwind.css") {
///     Ok(css) => println!("Generated CSS: {}", css),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn download_and_run(version: &str, fast: bool, tailwind_css_input: &str) -> Result<String> {
    run_with_local(&download(version, fast)?, tailwind_css_input)
}

/// Run the globally-installed `tailwind` executable.
pub fn run_with_global(tailwind_css_input: &str) -> Result<String> {
    let (shell, flag) = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    run_tailwind_command(Command::new(shell).args([
        flag,
        &format!("tailwindcss --input {tailwind_css_input} --output -"),
    ]))
}

fn run_with_local(tailwind_executable: &Path, tailwind_css_input: &str) -> Result<String> {
    run_tailwind_command(Command::new(tailwind_executable.canonicalize()?).args([
        "--input",
        tailwind_css_input,
        "--output",
        "-",
    ]))
}

fn run_tailwind_command(command: &mut Command) -> Result<String> {
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

/// Download a version of Tailwind CSS's CLI
fn download(version: &str, fast: bool) -> Result<PathBuf> {
    let output_path = if cfg!(target_os = "windows") {
        PathBuf::from("tailwind.exe")
    } else {
        PathBuf::from("tailwind")
    };

    // Check if executable exists and has correct version
    if output_path.exists() {
        if fast {
            // We skip the version check if fast mode is enabled
            return Ok(output_path);
        }

        let output = Command::new(output_path.canonicalize()?).output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let Some(first_line) = stdout.lines().next() else {
            return Err(TailwindError::VersionRead);
        };
        if first_line.contains(&format!("v{version}")) {
            return Ok(output_path);
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

    Ok(output_path)
}
