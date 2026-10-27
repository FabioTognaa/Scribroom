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
    let path = default_model_path(models_dir);
    if path.is_file() {
        return Ok(path);
    }
    let known = known_model(DEFAULT_MODEL_FILE).expect("the default model is documented");
    fetch(url, &path, known.size_bytes, known.sha256)?;
    Ok(path)
}

/// Download `url` to `destination`, verifying size and SHA-256 before the final
/// name appears. The transfer goes through `curl`, whose progress bar, retries
/// and resume support come for free.
fn fetch(url: &str, destination: &Path, expected_size: u64, expected_sha256: &str) -> Result<()> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    crate::validation::ensure_disk_space(
        parent,
        expected_size.saturating_add(crate::validation::SESSION_DISK_HEADROOM_BYTES),
    )?;

    // ponytail: the per-process `.part` keeps two concurrent runs from writing
    // the same file; the price is that a killed download restarts from zero.
    let partial = destination.with_extension(format!("{}.part", std::process::id()));
    eprintln!(
        "downloading the default model ({} MB) from {url}",
        expected_size / 1_000_000
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
    let status = curl.arg(url).status().map_err(|error| Error::ModelDownload {
        url: url.to_owned(),
        reason: if error.kind() == std::io::ErrorKind::NotFound {
            "curl was not found; install curl or pass --model <path>".to_owned()
        } else {
            error.to_string()
        },
    })?;
    if !status.success() {
        let _ = std::fs::remove_file(&partial);
        return Err(Error::ModelDownload {
            url: url.to_owned(),
            reason: format!("curl exited with {status}"),
        });
    }

    let actual = crate::asr::sha256_file(&partial)?;
    if actual != expected_sha256 {
        let _ = std::fs::remove_file(&partial);
        return Err(Error::ModelChecksum {
            expected: expected_sha256.to_owned(),
            actual,
        });
    }

    if destination.exists() {
        // Another process finished first: keep its file and verify it.
        let _ = std::fs::remove_file(&partial);
        let identity = crate::asr::model_identity(destination)?;
        if identity.sha256 != expected_sha256 {
            return Err(Error::ModelChecksum {
                expected: expected_sha256.to_owned(),
                actual: identity.sha256,
            });
        }
        return Ok(());
    }

    std::fs::rename(&partial, destination)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL_BYTES: &[u8] = b"model bytes";
    const MODEL_BYTES_SHA256: &str =
        "9cb7487000bc86ac36ce83c4acfabe8878552be99572a6770f65ab1d048a5c48";

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
    fn fetch_installs_a_verified_file() {
        let temporary = tempfile::tempdir().expect("temp directory should be created");
        let source = temporary.path().join("source.bin");
        std::fs::write(&source, MODEL_BYTES).expect("fixture should be written");
        let destination = temporary
            .path()
            .join("models")
            .join("ggml-large-v3-turbo-q8_0.bin");

        fetch(
            &file_url(&source),
            &destination,
            MODEL_BYTES.len() as u64,
            MODEL_BYTES_SHA256,
        )
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
    fn fetch_removes_the_partial_file_when_the_hash_does_not_match() {
        let temporary = tempfile::tempdir().expect("temp directory should be created");
        let source = temporary.path().join("source.bin");
        std::fs::write(&source, MODEL_BYTES).expect("fixture should be written");
        let destination = temporary.path().join("ggml-large-v3-turbo-q8_0.bin");

        let error = fetch(&file_url(&source), &destination, MODEL_BYTES.len() as u64, &"0".repeat(64))
            .expect_err("a wrong hash must fail");

        assert!(matches!(error, Error::ModelChecksum { .. }), "{error}");
        assert!(!destination.exists(), "no model must be left behind");
        assert!(
            part_files(temporary.path()).is_empty(),
            "the partial file must be removed"
        );
    }

    #[test]
    fn fetch_keeps_an_existing_destination() {
        let temporary = tempfile::tempdir().expect("temp directory should be created");
        let source = temporary.path().join("source.bin");
        std::fs::write(&source, MODEL_BYTES).expect("fixture should be written");
        let destination = temporary.path().join("ggml-large-v3-turbo-q8_0.bin");
        std::fs::write(&destination, MODEL_BYTES).expect("fixture should be written");

        fetch(
            &file_url(&source),
            &destination,
            MODEL_BYTES.len() as u64,
            MODEL_BYTES_SHA256,
        )
        .expect("an existing verified file should be kept");

        assert_eq!(
            std::fs::read(&destination).expect("model should be readable"),
            MODEL_BYTES
        );
        assert!(part_files(temporary.path()).is_empty());
    }

    #[test]
    fn fetch_refuses_when_disk_space_is_insufficient() {
        let temporary = tempfile::tempdir().expect("temp directory should be created");
        let source = temporary.path().join("source.bin");
        std::fs::write(&source, MODEL_BYTES).expect("fixture should be written");
        let destination = temporary.path().join("ggml-large-v3-turbo-q8_0.bin");

        let error = fetch(&file_url(&source), &destination, u64::MAX, MODEL_BYTES_SHA256)
            .expect_err("an impossible size must fail before downloading");

        assert!(matches!(error, Error::InsufficientDiskSpace { .. }), "{error}");
        assert!(!destination.exists());
    }

    #[test]
    fn ensure_default_model_accepts_an_existing_file_without_downloading() {
        let temporary = tempfile::tempdir().expect("temp directory should be created");
        let path = default_model_path(temporary.path());
        std::fs::write(&path, MODEL_BYTES).expect("fixture should be written");

        let resolved = ensure_default_model(temporary.path(), "file:///definitely/missing")
            .expect("an existing model should not trigger a download");

        assert_eq!(resolved, path);
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
