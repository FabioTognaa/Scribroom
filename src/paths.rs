//! The application data directory and the one place that decides where
//! sessions and models live.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{Error, Result};

/// Product name, also the directory name inside the platform data root.
pub const PRODUCT: &str = "Scrib";

const SESSIONS_DIR: &str = "sessions";
const MODELS_DIR: &str = "models";
const CONFIG_FILE: &str = "config.json";

/// Every directory the application reads or writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDirs {
    pub data: PathBuf,
    pub sessions: PathBuf,
    pub models: PathBuf,
}

/// The optional `config.json` the user writes in the data root. Paths are
/// already validated and `~` is already expanded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub sessions_dir: Option<PathBuf>,
    pub models_dir: Option<PathBuf>,
}

/// Environment variable that moves the data root for development.
pub const DATA_DIR_ENV: &str = "SCRIB_DATA_DIR";

/// Resolve every directory from the already-decided default data root, the
/// explicit `--sessions-dir` flag, the `SCRIB_DATA_DIR` value and the parsed
/// config. No I/O, so tests never touch `$HOME`. Precedence: flag, then
/// environment, then config, then the default under the root. A set environment
/// wins over the config, which is ignored entirely, and its root may be relative
/// (`.` reproduces the old source-tree layout).
#[must_use]
pub fn resolve(
    default_root: &Path,
    sessions_flag: Option<&Path>,
    environment: Option<&Path>,
    config: Option<&Config>,
) -> DataDirs {
    let data = environment.unwrap_or(default_root);
    let config = config.filter(|_| environment.is_none());
    DataDirs {
        data: data.to_path_buf(),
        sessions: sessions_flag
            .map(Path::to_path_buf)
            .or_else(|| config.and_then(|config| config.sessions_dir.clone()))
            .unwrap_or_else(|| data.join(SESSIONS_DIR)),
        models: config
            .and_then(|config| config.models_dir.clone())
            .unwrap_or_else(|| data.join(MODELS_DIR)),
    }
}

/// Parse the config text. Pure: the caller passes the file path (for error
/// messages) and the home directory used to expand `~`. Unknown keys are
/// ignored; malformed JSON is an error, never a silent fallback.
pub fn parse_config(text: &str, path: &Path, home: &Path) -> Result<Config> {
    let file: ConfigFile = serde_json::from_str(text).map_err(|error| Error::InvalidConfig {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    Ok(Config {
        sessions_dir: file
            .sessions_dir
            .map(|value| expand(value, "sessions_dir", path, home))
            .transpose()?,
        models_dir: file
            .models_dir
            .map(|value| expand(value, "models_dir", path, home))
            .transpose()?,
    })
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    sessions_dir: Option<PathBuf>,
    models_dir: Option<PathBuf>,
}

/// Accept an absolute path or one starting with `~`; a relative path has no
/// obvious meaning, so it is rejected naming the config file to fix.
fn expand(value: PathBuf, key: &str, config: &Path, home: &Path) -> Result<PathBuf> {
    if let Ok(rest) = value.strip_prefix("~") {
        return Ok(home.join(rest));
    }
    if value.is_absolute() {
        return Ok(value);
    }
    Err(Error::InvalidConfig {
        path: config.to_path_buf(),
        reason: format!(
            "{key} must be an absolute path or start with `~`, got {}",
            value.display()
        ),
    })
}

/// Read `config.json` from the data root. A missing file is `None` and is never
/// created; any other read failure or invalid content is an error.
pub fn load_config(data_root: &Path, home: &Path) -> Result<Option<Config>> {
    let path = data_root.join(CONFIG_FILE);
    match std::fs::read_to_string(&path) {
        Ok(text) => parse_config(&text, &path, home).map(Some),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

/// Default data root for the current platform.
pub fn platform_data_root() -> Result<PathBuf> {
    let home = home_directory()?;
    Ok(default_root(&home))
}

/// User home directory, also used to expand a `~` in the config.
pub fn home_directory() -> Result<PathBuf> {
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

    const DATA: &str = "/data/Scrib";
    const CONFIG_PATH: &str = "/data/Scrib/config.json";
    const HOME: &str = "/Users/test";

    /// Compose the two pure steps the way `data_dirs` does: parse the config
    /// text, then resolve. Nothing here touches `$HOME` or the disk.
    fn dirs(
        config: Option<&str>,
        environment: Option<&Path>,
        sessions_flag: Option<&Path>,
    ) -> Result<DataDirs> {
        let config = config
            .map(|text| parse_config(text, Path::new(CONFIG_PATH), Path::new(HOME)))
            .transpose()?;
        Ok(resolve(
            Path::new(DATA),
            sessions_flag,
            environment,
            config.as_ref(),
        ))
    }

    #[test]
    fn precedence_is_flag_then_environment_then_config_then_default() {
        let config =
            r#"{"sessions_dir": "/Volumes/ext/sessions", "models_dir": "/Volumes/ext/models"}"#;
        let environment = Path::new("/tmp/dev-root");
        let flag = Path::new("/tmp/one-off");

        let all = dirs(Some(config), Some(environment), Some(flag))
            .expect("every source is valid at once");
        assert_eq!(all.data, environment);
        assert_eq!(all.sessions, flag);
        assert_eq!(all.models, Path::new("/tmp/dev-root/models"));

        let environment_over_config =
            dirs(Some(config), Some(environment), None).expect("environment beats config");
        assert_eq!(
            environment_over_config.sessions,
            Path::new("/tmp/dev-root/sessions")
        );
        assert_eq!(
            environment_over_config.models,
            Path::new("/tmp/dev-root/models")
        );

        let config_over_default = dirs(Some(config), None, None).expect("config beats the default");
        assert_eq!(config_over_default.data, Path::new(DATA));
        assert_eq!(
            config_over_default.sessions,
            Path::new("/Volumes/ext/sessions")
        );
    }

    #[test]
    fn environment_root_reproduces_the_source_layout() {
        let dirs = dirs(None, Some(Path::new(".")), None).expect("a relative root is valid");

        assert_eq!(dirs.data, Path::new("."));
        assert_eq!(dirs.sessions, Path::new("./sessions"));
        assert_eq!(dirs.models, Path::new("./models"));
    }

    #[test]
    fn relative_environment_root_stays_relative() {
        let dirs = dirs(None, Some(Path::new("devdata")), None).expect("a relative root is valid");

        assert_eq!(dirs.sessions, Path::new("devdata/sessions"));
        assert_eq!(dirs.models, Path::new("devdata/models"));
    }

    #[test]
    fn explicit_flag_stays_relative_under_the_environment_root() {
        let dirs = dirs(None, Some(Path::new(".")), Some(Path::new("out")))
            .expect("a relative flag is valid");

        assert_eq!(dirs.sessions, Path::new("out"));
    }

    #[test]
    fn without_the_environment_the_platform_default_is_the_root() {
        let platform = Path::new("/Users/test/Library/Application Support/Scrib");
        let dirs = resolve(platform, None, None, None);

        assert_eq!(dirs.data, platform);
        assert_eq!(dirs.sessions, platform.join("sessions"));
        assert_eq!(dirs.models, platform.join("models"));
    }

    #[test]
    fn sessions_and_models_live_under_the_data_root() {
        let dirs = dirs(None, None, None).expect("no config is not an error");

        assert_eq!(dirs.data, Path::new(DATA));
        assert_eq!(dirs.sessions, Path::new("/data/Scrib/sessions"));
        assert_eq!(dirs.models, Path::new("/data/Scrib/models"));
    }

    #[test]
    fn one_config_key_moves_only_that_directory() {
        let models = dirs(Some(r#"{"models_dir": "/Volumes/ext/models"}"#), None, None)
            .expect("a models_dir key is valid");
        assert_eq!(models.models, Path::new("/Volumes/ext/models"));
        assert_eq!(models.sessions, Path::new("/data/Scrib/sessions"));

        let sessions = dirs(
            Some(r#"{"sessions_dir": "/Volumes/ext/sessions"}"#),
            None,
            None,
        )
        .expect("a sessions_dir key is valid");
        assert_eq!(sessions.sessions, Path::new("/Volumes/ext/sessions"));
        assert_eq!(sessions.models, Path::new("/data/Scrib/models"));
    }

    #[test]
    fn explicit_sessions_flag_wins_over_the_default() {
        let dirs =
            dirs(None, None, Some(Path::new("elsewhere"))).expect("no config is not an error");

        assert_eq!(dirs.sessions, Path::new("elsewhere"));
        assert_eq!(dirs.models, Path::new("/data/Scrib/models"));
    }

    #[test]
    fn explicit_sessions_flag_wins_over_the_config() {
        let dirs = dirs(
            Some(r#"{"sessions_dir": "/Volumes/ext/sessions"}"#),
            None,
            Some(Path::new("/tmp/one-off")),
        )
        .expect("a valid config with a flag is not an error");

        assert_eq!(dirs.sessions, Path::new("/tmp/one-off"));
    }

    #[test]
    fn tilde_is_expanded_to_the_given_home() {
        let dirs = dirs(
            Some(r#"{"sessions_dir": "~/Documenti/sessioni", "models_dir": "~"}"#),
            None,
            None,
        )
        .expect("a tilde path is valid");

        assert_eq!(dirs.sessions, Path::new("/Users/test/Documenti/sessioni"));
        assert_eq!(dirs.models, Path::new("/Users/test"));
    }

    #[test]
    fn relative_config_path_is_rejected_citing_the_config_file() {
        let error = dirs(Some(r#"{"sessions_dir": "sessioni"}"#), None, None)
            .expect_err("a relative path must be rejected")
            .to_string();

        assert!(error.contains(CONFIG_PATH), "{error}");
        assert!(error.contains("sessioni"), "{error}");
    }

    #[test]
    fn malformed_json_is_rejected_citing_the_config_file() {
        let error = dirs(Some(r#"{"sessions_dir": "#), None, None)
            .expect_err("malformed JSON must be rejected")
            .to_string();

        assert!(error.contains(CONFIG_PATH), "{error}");
    }

    #[test]
    fn empty_config_file_is_rejected() {
        for text in ["", "  \n"] {
            let error = dirs(Some(text), None, None)
                .expect_err("an empty config must be rejected")
                .to_string();
            assert!(error.contains(CONFIG_PATH), "{error}");
        }
    }

    #[test]
    fn unknown_config_keys_are_ignored() {
        let dirs = dirs(
            Some(r#"{"sessions_dir": "/Volumes/ext/sessions", "future_key": [1, 2]}"#),
            None,
            None,
        )
        .expect("an unknown key must not fail the parse");

        assert_eq!(dirs.sessions, Path::new("/Volumes/ext/sessions"));
    }

    #[test]
    fn load_config_reads_the_file_and_never_creates_it() {
        let temporary = tempfile::tempdir().expect("temp directory should be created");
        let data = temporary.path().join("Scrib");

        let absent = load_config(&data, Path::new(HOME)).expect("a missing config is not an error");
        assert_eq!(absent, None);
        assert!(
            !data.exists(),
            "reading the config must not create anything"
        );

        std::fs::create_dir(&data).expect("data directory should be created");
        std::fs::write(data.join("config.json"), r#"{"models_dir": "~/modelli"}"#)
            .expect("fixture should be written");

        let config = load_config(&data, Path::new(HOME))
            .expect("a valid config should load")
            .expect("the file exists");
        assert_eq!(
            config.models_dir,
            Some(PathBuf::from("/Users/test/modelli"))
        );
        assert_eq!(config.sessions_dir, None);
        assert_eq!(
            load_config(&data, Path::new(HOME))
                .expect("a valid config should load")
                .expect("the file exists"),
            config,
            "loading must be repeatable and leave the file untouched"
        );
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
