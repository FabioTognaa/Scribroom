//! The application data directory and the one place that decides where
//! sessions and models live.

use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// Product name, also the directory name inside the platform data root.
pub const PRODUCT: &str = "Scrib";

const SESSIONS_DIR: &str = "sessions";
const MODELS_DIR: &str = "models";

/// Every directory the application reads or writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDirs {
    pub data: PathBuf,
    pub sessions: PathBuf,
    pub models: PathBuf,
}

/// Resolve every directory from the already-decided data root and the
/// explicit `--sessions-dir` flag. No I/O, so tests never touch `$HOME`.
#[must_use]
pub fn resolve(data: &Path, sessions_flag: Option<&Path>) -> DataDirs {
    DataDirs {
        data: data.to_path_buf(),
        sessions: sessions_flag.map_or_else(|| data.join(SESSIONS_DIR), Path::to_path_buf),
        models: data.join(MODELS_DIR),
    }
}

/// Default data root for the current platform.
pub fn platform_data_root() -> Result<PathBuf> {
    let home = home_directory()?;
    Ok(default_root(&home))
}

fn home_directory() -> Result<PathBuf> {
    let variable = if cfg!(target_os = "windows") {
        "USERPROFILE"
    } else {
        "HOME"
    };
    std::env::var_os(variable)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or_else(|| Error::InvalidPath {
            label: "home directory",
            path: PathBuf::new(),
            reason: format!("{variable} is not set to an absolute path"),
        })
}

#[cfg(target_os = "linux")]
fn default_root(home: &Path) -> PathBuf {
    match std::env::var_os("XDG_DATA_HOME").map(PathBuf::from) {
        Some(root) if root.is_absolute() => root.join(PRODUCT),
        _ => home.join(".local").join("share").join(PRODUCT),
    }
}

#[cfg(target_os = "windows")]
fn default_root(home: &Path) -> PathBuf {
    match std::env::var_os("APPDATA").map(PathBuf::from) {
        Some(root) if root.is_absolute() => root.join(PRODUCT),
        _ => home.join("AppData").join("Roaming").join(PRODUCT),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn default_root(home: &Path) -> PathBuf {
    home.join("Library")
        .join("Application Support")
        .join(PRODUCT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sessions_and_models_live_under_the_data_root() {
        let dirs = resolve(Path::new("/data/Scrib"), None);

        assert_eq!(dirs.data, Path::new("/data/Scrib"));
        assert_eq!(dirs.sessions, Path::new("/data/Scrib/sessions"));
        assert_eq!(dirs.models, Path::new("/data/Scrib/models"));
    }

    #[test]
    fn explicit_sessions_flag_wins_over_the_default() {
        let dirs = resolve(Path::new("/data/Scrib"), Some(Path::new("elsewhere")));

        assert_eq!(dirs.sessions, Path::new("elsewhere"));
        assert_eq!(dirs.models, Path::new("/data/Scrib/models"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn platform_default_is_application_support() {
        assert_eq!(
            default_root(Path::new("/Users/test")),
            Path::new("/Users/test/Library/Application Support/Scrib")
        );
    }

    #[test]
    fn platform_root_ends_with_the_product_name() {
        let root = platform_data_root().expect("the test environment has a home directory");

        assert_eq!(root.file_name(), Some(std::ffi::OsStr::new(PRODUCT)));
        assert!(root.is_absolute());
    }
}
