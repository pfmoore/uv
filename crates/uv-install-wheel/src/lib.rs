//! Takes a wheel and installs it into a venv.

use std::io;
use std::path::PathBuf;

use platform_info::PlatformInfoError;
use thiserror::Error;
use zip::result::ZipError;

use uv_fs::Simplified;
use uv_normalize::PackageName;
use uv_pep440::Version;
use uv_platform_tags::{Arch, Os};
use uv_pypi_types::Scheme;

pub use install::install_wheel;
pub use linker::{LinkMode, Locks};
pub use uninstall::{uninstall_egg, uninstall_legacy_editable, uninstall_wheel, Uninstall};
pub use wheel::{parse_wheel_file, read_record_file, LibKind};

mod install;
mod linker;
mod record;
mod script;
mod uninstall;
mod wheel;

/// The layout of the target environment into which a wheel can be installed.
#[derive(Debug, Clone)]
pub struct Layout {
    /// The Python interpreter, as returned by `sys.executable`.
    pub sys_executable: PathBuf,
    /// The Python version, as returned by `sys.version_info`.
    pub python_version: (u8, u8),
    /// The `os.name` value for the current platform.
    pub os_name: String,
    /// The [`Scheme`] paths for the interpreter.
    pub scheme: Scheme,
}

/// Note: The caller is responsible for adding the path of the wheel we're installing.
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    /// Custom error type to add a path to error reading a file from a zip
    #[error("Failed to reflink {} to {}", from.user_display(), to.user_display())]
    Reflink {
        from: PathBuf,
        to: PathBuf,
        #[source]
        err: io::Error,
    },
    /// Tags/metadata didn't match platform
    #[error("The wheel is incompatible with the current platform {os} {arch}")]
    IncompatibleWheel { os: Os, arch: Arch },
    /// The wheel is broken
    #[error("The wheel is invalid: {0}")]
    InvalidWheel(String),
    /// Doesn't follow file name schema
    #[error(transparent)]
    InvalidWheelFileName(#[from] uv_distribution_filename::WheelFilenameError),
    /// The caller must add the name of the zip file (See note on type).
    #[error("Failed to read {0} from zip file")]
    Zip(String, #[source] ZipError),
    #[error("Failed to run Python subcommand")]
    PythonSubcommand(#[source] io::Error),
    #[error("Failed to move data files")]
    WalkDir(#[from] walkdir::Error),
    #[error("RECORD file doesn't match wheel contents: {0}")]
    RecordFile(String),
    #[error("RECORD file is invalid")]
    RecordCsv(#[from] csv::Error),
    #[error("Broken virtual environment: {0}")]
    BrokenVenv(String),
    #[error(
        "Unable to create Windows launcher for: {0} (only x86_64, x86, and arm64 are supported)"
    )]
    UnsupportedWindowsArch(&'static str),
    #[error("Unable to create Windows launcher on non-Windows platform")]
    NotWindows,
    #[error("Failed to detect the current platform")]
    PlatformInfo(#[source] PlatformInfoError),
    #[error("Invalid version specification, only none or == is supported")]
    Pep440,
    #[error("Invalid direct_url.json")]
    DirectUrlJson(#[from] serde_json::Error),
    #[error("Cannot uninstall package; `RECORD` file not found at: {}", _0.user_display())]
    MissingRecord(PathBuf),
    #[error("Cannot uninstall package; `top_level.txt` file not found at: {}", _0.user_display())]
    MissingTopLevel(PathBuf),
    #[error("Invalid wheel size")]
    InvalidSize,
    #[error("Invalid package name")]
    InvalidName(#[from] uv_normalize::InvalidNameError),
    #[error("Invalid package version")]
    InvalidVersion(#[from] uv_pep440::VersionParseError),
    #[error("Wheel package name does not match filename: {0} != {1}")]
    MismatchedName(PackageName, PackageName),
    #[error("Wheel version does not match filename: {0} != {1}")]
    MismatchedVersion(Version, Version),
    #[error("Invalid egg-link")]
    InvalidEggLink(PathBuf),
    #[error(transparent)]
    LauncherError(#[from] uv_trampoline_builder::Error),
}
