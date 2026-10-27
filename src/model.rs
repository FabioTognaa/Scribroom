//! The Whisper model the application ships and the table of documented artifacts.
//!
//! The product has one model: `large-v3-turbo` in Q8_0, resolved under the data
//! directory's `models` folder. The known-model table lets `doctor` verify that a
//! file is the documented artifact (name, size and SHA-256) instead of trusting
//! any non-empty file that happens to share the name.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// File name of the model the application uses when no `--model` is given.
pub const DEFAULT_MODEL_FILE: &str = "ggml-large-v3-turbo-q8_0.bin";

/// Path of the default model inside `models_dir`.
#[must_use]
pub fn default_model_path(models_dir: &Path) -> PathBuf {
    models_dir.join(DEFAULT_MODEL_FILE)
}

/// A model file published by the `ggerganov/whisper.cpp` GGML repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownModel {
    pub file_name: &'static str,
    pub size_bytes: u64,
    pub sha256: &'static str,
}

/// Models with a documented size and SHA-256, verified by `doctor`.
///
/// Values come from the Hugging Face `ggerganov/whisper.cpp` repository: the LFS
/// object id of each file is its SHA-256.
pub const KNOWN_MODELS: &[KnownModel] = &[
    // Whisper small.
    KnownModel {
        file_name: "ggml-small-q5_1.bin",
        size_bytes: 190_085_487,
        sha256: "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb",
    },
    KnownModel {
        file_name: "ggml-small-q8_0.bin",
        size_bytes: 264_464_607,
        sha256: "49c8fb02b65e6049d5fa6c04f81f53b867b5ec9540406812c643f177317f779f",
    },
    KnownModel {
        file_name: "ggml-small.bin",
        size_bytes: 487_601_967,
        sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
    },
    // Whisper medium.
    KnownModel {
        file_name: "ggml-medium-q5_0.bin",
        size_bytes: 539_212_467,
        sha256: "19fea4b380c3a618ec4723c3eef2eb785ffba0d0538cf43f8f235e7b3b34220f",
    },
    KnownModel {
        file_name: "ggml-medium-q8_0.bin",
        size_bytes: 823_369_779,
        sha256: "42a1ffcbe4167d224232443396968db4d02d4e8e87e213d3ee2e03095dea6502",
    },
    KnownModel {
        file_name: "ggml-medium.bin",
        size_bytes: 1_533_763_059,
        sha256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
    },
    // Whisper large-v3-turbo.
    KnownModel {
        file_name: "ggml-large-v3-turbo-q5_0.bin",
        size_bytes: 574_041_195,
        sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
    },
    KnownModel {
        file_name: "ggml-large-v3-turbo-q8_0.bin",
        size_bytes: 874_188_075,
        sha256: "317eb69c11673c9de1e1f0d459b253999804ec71ac4c23c17ecf5fbe24e259a1",
    },
    KnownModel {
        file_name: "ggml-large-v3-turbo.bin",
        size_bytes: 1_624_555_275,
        sha256: "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
    },
    // Whisper large-v3.
    KnownModel {
        file_name: "ggml-large-v3-q5_0.bin",
        size_bytes: 1_081_140_203,
        sha256: "d75795ecff3f83b5faa89d1900604ad8c780abd5739fae406de19f23ecd98ad1",
    },
    KnownModel {
        file_name: "ggml-large-v3.bin",
        size_bytes: 3_095_033_483,
        sha256: "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2",
    },
];

/// Look up the documented artifact for a model file name, if it is known.
#[must_use]
pub fn known_model(file_name: &str) -> Option<&'static KnownModel> {
    KNOWN_MODELS
        .iter()
        .find(|model| model.file_name == file_name)
}

/// Source of the default model.
pub const DEFAULT_MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q8_0.bin";

/// Path of the default model, downloading and verifying it when missing.
///
/// An existing file is trusted as-is: `doctor` verifies its identity, and a
/// user-supplied `--model` never reaches here.
pub fn ensure_default_model(models_dir: &Path, url: &str) -> Result<PathBuf> {
    let known = known_model(DEFAULT_MODEL_FILE).expect("the default model is documented");
    let path = default_model_path(models_dir);
    if path.is_file() {
        let identity = crate::asr::model_identity(&path)?;
        if identity.sha256 != known.sha256 {
            return Err(checksum_mismatch(known, identity.sha256));
        }
        return Ok(path);
    }
    download_verified_model(url, &path, known)?;
    Ok(path)
}

fn checksum_mismatch(expected: &KnownModel, actual: String) -> Error {
    Error::ModelChecksum {
        expected: expected.sha256.to_owned(),
        actual,
    }
}

/// Exclusive lock for the model download. `flock` releases it when the process
/// exits, so a crashed run leaves no stale lock behind.
#[cfg(unix)]
fn lock_download(destination: &Path) -> Result<std::fs::File> {
    use std::os::unix::io::AsRawFd;

    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(destination.with_extension("lock"))?;
    // SAFETY: `file` owns a valid descriptor for the lifetime of the call.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err(Error::ModelDownloadBusy);
    }
    Ok(file)
}

#[cfg(not(unix))]
fn lock_download(_destination: &Path) -> Result<()> {
    Ok(())
}

/// Download `url` to `destination`, verifying size and SHA-256 before the final
/// name appears. `curl` provides the transport, the progress bar, retries and
/// resume; the lock keeps two runs from writing the same partial file.
fn download_verified_model(url: &str, destination: &Path, expected: &KnownModel) -> Result<()> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    // The partial name is stable so an interrupted download resumes with `-C -`.
    let partial = destination.with_extension("part");
    let resumed_bytes = std::fs::metadata(&partial).map_or(0, |metadata| metadata.len());
    crate::validation::ensure_disk_space(
        parent,
        expected
            .size_bytes
            .saturating_sub(resumed_bytes)
            .saturating_add(crate::validation::SESSION_DISK_HEADROOM_BYTES),
    )?;
    let _lock = lock_download(destination)?;

    eprintln!(
        "downloading {} ({} MB) from {url}",
        expected.file_name,
        expected.size_bytes / 1_000_000
    );

    let mut curl = std::process::Command::new("curl");
    curl.arg("--fail")
        .arg("--location")
        .arg("--retry")
        .arg("3")
        .arg("--retry-delay")
        .arg("2")
        .arg("--continue-at")
        .arg("-")
        .arg("--output")
        .arg(&partial);
    if std::io::stderr().is_terminal() {
        curl.arg("--progress-bar");
    } else {
        curl.arg("--silent").arg("--show-error");
    }
    let status = curl
        .arg(url)
        .status()
        .map_err(|error| Error::ModelDownload {
            url: url.to_owned(),
            reason: if error.kind() == std::io::ErrorKind::NotFound {
                "curl was not found; install curl or pass --model <path>".to_owned()
            } else {
                error.to_string()
            },
        })?;
    if !status.success() {
        // A partial at least as large as the artifact can only be stale (the
        // release changed size); `-C -` would fail on it forever, so discard it.
        let stale =
            std::fs::metadata(&partial).is_ok_and(|metadata| metadata.len() >= expected.size_bytes);
        if stale {
            let _ = std::fs::remove_file(&partial);
        }
        return Err(Error::ModelDownload {
            url: url.to_owned(),
            reason: if stale {
                format!(
                    "curl exited with {status}; the stale partial download was discarded, retry"
                )
            } else {
                format!("curl exited with {status}; check the connection or pass --model <path>")
            },
        });
    }

    let actual = crate::asr::sha256_file(&partial)?;
    if actual != expected.sha256 {
        let _ = std::fs::remove_file(&partial);
        return Err(checksum_mismatch(expected, actual));
    }

    if destination.exists() {
        // Another process finished first: keep its file and verify it.
        let _ = std::fs::remove_file(&partial);
        let identity = crate::asr::model_identity(destination)?;
        if identity.sha256 != expected.sha256 {
            return Err(checksum_mismatch(expected, identity.sha256));
        }
        return Ok(());
    }

    std::fs::rename(&partial, destination)?;
    // The hash is already known; cache it so the first identity call is free.
    let _ = crate::asr::remember_model_identity(destination, expected.sha256);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL_BYTES: &[u8] = b"model bytes";
    const MODEL_BYTES_SHA256: &str =
        "9cb7487000bc86ac36ce83c4acfabe8878552be99572a6770f65ab1d048a5c48";
    const MODEL: KnownModel = KnownModel {
        file_name: "test.bin",
        size_bytes: MODEL_BYTES.len() as u64,
        sha256: MODEL_BYTES_SHA256,
    };
    const WRONG_MODEL: KnownModel = KnownModel {
        file_name: "test.bin",
        size_bytes: MODEL_BYTES.len() as u64,
        sha256: "0000000000000000000000000000000000000000000000000000000000000000",
    };
    const HUGE_MODEL: KnownModel = KnownModel {
        file_name: "test.bin",
        size_bytes: u64::MAX,
        sha256: MODEL_BYTES_SHA256,
    };

    fn file_url(path: &Path) -> String {
        format!("file://{}", path.display())
    }

    fn part_files(directory: &Path) -> Vec<PathBuf> {
        std::fs::read_dir(directory)
            .expect("directory should be readable")
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("part"))
            .collect()
    }

    #[test]
    fn the_default_model_resolves_under_the_models_directory() {
        let models = Path::new("/data/Scribroom/models");

        assert_eq!(
            default_model_path(models),
            PathBuf::from("/data/Scribroom/models/ggml-large-v3-turbo-q8_0.bin")
        );
    }

    #[test]
    fn download_installs_a_verified_file() {
        let temporary = tempfile::tempdir().expect("temp directory should be created");
        let source = temporary.path().join("source.bin");
        std::fs::write(&source, MODEL_BYTES).expect("fixture should be written");
        let destination = temporary.path().join("models").join(DEFAULT_MODEL_FILE);

        download_verified_model(&file_url(&source), &destination, &MODEL)
            .expect("a matching file should install");

        assert_eq!(
            std::fs::read(&destination).expect("model should be readable"),
            MODEL_BYTES
        );
        assert!(
            part_files(&temporary.path().join("models")).is_empty(),
            "no partial file must remain"
        );
    }

    #[test]
    fn download_resumes_a_partial_file() {
        let temporary = tempfile::tempdir().expect("temp directory should be created");
        let source = temporary.path().join("source.bin");
        std::fs::write(&source, MODEL_BYTES).expect("fixture should be written");
        let destination = temporary.path().join(DEFAULT_MODEL_FILE);
        std::fs::write(destination.with_extension("part"), &MODEL_BYTES[..6])
            .expect("partial fixture should be written");

        download_verified_model(&file_url(&source), &destination, &MODEL)
            .expect("a partial file should resume");

        assert_eq!(
            std::fs::read(&destination).expect("model should be readable"),
            MODEL_BYTES
        );
        assert!(part_files(temporary.path()).is_empty());
    }

    #[test]
    fn download_removes_the_partial_file_when_the_hash_does_not_match() {
        let temporary = tempfile::tempdir().expect("temp directory should be created");
        let source = temporary.path().join("source.bin");
        std::fs::write(&source, MODEL_BYTES).expect("fixture should be written");
        let destination = temporary.path().join(DEFAULT_MODEL_FILE);

        let error = download_verified_model(&file_url(&source), &destination, &WRONG_MODEL)
            .expect_err("a wrong hash must fail");

        assert!(matches!(error, Error::ModelChecksum { .. }), "{error}");
        assert!(!destination.exists(), "no model must be left behind");
        assert!(
            part_files(temporary.path()).is_empty(),
            "the partial file must be removed"
        );
    }

    #[test]
    fn download_keeps_an_existing_destination() {
        let temporary = tempfile::tempdir().expect("temp directory should be created");
        let source = temporary.path().join("source.bin");
        std::fs::write(&source, MODEL_BYTES).expect("fixture should be written");
        let destination = temporary.path().join(DEFAULT_MODEL_FILE);
        std::fs::write(&destination, MODEL_BYTES).expect("fixture should be written");

        download_verified_model(&file_url(&source), &destination, &MODEL)
            .expect("an existing verified file should be kept");

        assert_eq!(
            std::fs::read(&destination).expect("model should be readable"),
            MODEL_BYTES
        );
        assert!(part_files(temporary.path()).is_empty());
    }

    #[test]
    fn download_refuses_when_disk_space_is_insufficient() {
        let temporary = tempfile::tempdir().expect("temp directory should be created");
        let source = temporary.path().join("source.bin");
        std::fs::write(&source, MODEL_BYTES).expect("fixture should be written");
        let destination = temporary.path().join(DEFAULT_MODEL_FILE);

        let error = download_verified_model(&file_url(&source), &destination, &HUGE_MODEL)
            .expect_err("an impossible size must fail before downloading");

        assert!(
            matches!(error, Error::InsufficientDiskSpace { .. }),
            "{error}"
        );
        assert!(!destination.exists());
    }

    #[test]
    fn ensure_default_model_rejects_an_unverified_existing_file() {
        let temporary = tempfile::tempdir().expect("temp directory should be created");
        let path = default_model_path(temporary.path());
        std::fs::write(&path, MODEL_BYTES).expect("fixture should be written");

        // The unreachable URL proves the rejection happens before any download.
        let error = ensure_default_model(temporary.path(), "file:///definitely/missing")
            .expect_err("an existing file must still match the documented model");

        assert!(matches!(error, Error::ModelChecksum { .. }), "{error}");
    }

    #[test]
    fn documented_v1_model_is_known() {
        let model = known_model("ggml-small-q5_1.bin").expect("the v1 model is documented");

        assert_eq!(model.size_bytes, 190_085_487);
        assert_eq!(
            model.sha256,
            "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb"
        );
    }

    #[test]
    fn unknown_model_is_not_in_the_table() {
        assert!(known_model("custom-model.bin").is_none());
    }
}
