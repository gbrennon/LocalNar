use std::path::{Path, PathBuf};

use application::errors::LibraryError;
use application::ports::outbound::ModelLibraryPort;
use domain::{ByteLength, Checksum, InstalledModel, ModelArtifact, ModelSpec, ModelState};
use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;

/// Filesystem-backed storage for locally installed models.
///
/// Stores files in an `<owner>/<name>/<revision>/<filename>` hierarchy
/// under a configurable root directory.
#[derive(Debug, Clone)]
pub struct DiskModelLibrary {
    root: PathBuf,
}

impl DiskModelLibrary {
    /// Builds a library rooted at the given directory path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Resolves the default model cache directory from the environment or user cache.
    pub fn from_env() -> Self {
        let path = std::env::var("BARE_AI_MODELS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| Self::default_root());
        Self::new(path)
    }

    /// Returns the root directory path where models are stored.
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn default_root() -> PathBuf {
        dirs_path()
            .join(".cache")
            .join("bare-ai-server")
            .join("models")
    }

    fn model_file_path(&self, model: &ModelSpec) -> PathBuf {
        self.root
            .join(model.repository().identifier().owner())
            .join(model.repository().identifier().name())
            .join(model.repository().revision().as_str())
            .join(model.file().as_str())
    }

    fn checksum_file_path(&self, model: &ModelSpec) -> PathBuf {
        self.root
            .join(model.repository().identifier().owner())
            .join(model.repository().identifier().name())
            .join(model.repository().revision().as_str())
            .join(format!("{}.sha256", model.file().as_str()))
    }

    async fn compute_sha256(
        &self,
        path: &Path,
        model: &ModelSpec,
    ) -> Result<Checksum, LibraryError> {
        let mut file =
            tokio::fs::File::open(path)
                .await
                .map_err(|err| LibraryError::Unverifiable {
                    model: model.to_string(),
                    cause: err.to_string(),
                })?;

        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];

        loop {
            let bytes_read =
                file.read(&mut buffer)
                    .await
                    .map_err(|err| LibraryError::Unverifiable {
                        model: model.to_string(),
                        cause: err.to_string(),
                    })?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let digest: [u8; 32] = hasher.finalize().into();
        Ok(Checksum::from_bytes(digest))
    }
}

impl Default for DiskModelLibrary {
    fn default() -> Self {
        Self::from_env()
    }
}

impl ModelLibraryPort for DiskModelLibrary {
    async fn installed_state(&self, model: &ModelSpec) -> Result<ModelState, LibraryError> {
        let path = self.model_file_path(model);
        let file_exists =
            tokio::fs::try_exists(&path)
                .await
                .map_err(|err| LibraryError::Unreadable {
                    model: model.to_string(),
                    cause: err.to_string(),
                })?;

        if !file_exists {
            return Ok(ModelState::Missing);
        }

        let checksum_path = self.checksum_file_path(model);
        let sidecar_exists = tokio::fs::try_exists(&checksum_path).await.unwrap_or(false);

        if sidecar_exists
            && let Ok(content) = tokio::fs::read_to_string(&checksum_path).await
            && Checksum::parse(content.trim()).is_ok()
        {
            return Ok(ModelState::Verified);
        }

        Ok(ModelState::Downloaded)
    }

    async fn commit_artifact(
        &self,
        model: &ModelSpec,
        artifact: &ModelArtifact,
    ) -> Result<ModelState, LibraryError> {
        let destination = self.model_file_path(model);
        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| LibraryError::Unwritable {
                    model: model.to_string(),
                    cause: err.to_string(),
                })?;
        }

        let checksum_path = self.checksum_file_path(model);
        let _ = tokio::fs::remove_file(&checksum_path).await;

        let staged = artifact.staged_at();
        if staged != destination && tokio::fs::rename(staged, &destination).await.is_err() {
            tokio::fs::copy(staged, &destination).await.map_err(|err| {
                LibraryError::Unwritable {
                    model: model.to_string(),
                    cause: err.to_string(),
                }
            })?;
            let _ = tokio::fs::remove_file(staged).await;
        }

        Ok(ModelState::Downloaded)
    }

    async fn verify_integrity(
        &self,
        model: &ModelSpec,
        expected: Option<Checksum>,
    ) -> Result<ModelState, LibraryError> {
        let path = self.model_file_path(model);
        let file_exists =
            tokio::fs::try_exists(&path)
                .await
                .map_err(|err| LibraryError::Unverifiable {
                    model: model.to_string(),
                    cause: err.to_string(),
                })?;

        if !file_exists {
            return Ok(ModelState::Missing);
        }

        let Some(expected_checksum) = expected else {
            return Ok(ModelState::Downloaded);
        };

        let actual_checksum = self.compute_sha256(&path, model).await?;

        if actual_checksum == expected_checksum {
            let checksum_path = self.checksum_file_path(model);
            if let Some(parent) = checksum_path.parent() {
                tokio::fs::create_dir_all(parent).await.map_err(|err| {
                    LibraryError::Unwritable {
                        model: model.to_string(),
                        cause: err.to_string(),
                    }
                })?;
            }
            tokio::fs::write(&checksum_path, actual_checksum.to_hex())
                .await
                .map_err(|err| LibraryError::Unwritable {
                    model: model.to_string(),
                    cause: err.to_string(),
                })?;
            Ok(ModelState::Verified)
        } else {
            let checksum_path = self.checksum_file_path(model);
            let _ = tokio::fs::remove_file(&checksum_path).await;
            Ok(ModelState::IntegrityMismatch {
                expected: expected_checksum,
                actual: actual_checksum,
            })
        }
    }

    async fn locate(&self, model: &ModelSpec) -> Result<InstalledModel, LibraryError> {
        let path = self.model_file_path(model);
        let metadata =
            tokio::fs::metadata(&path)
                .await
                .map_err(|err| LibraryError::Unreadable {
                    model: model.to_string(),
                    cause: err.to_string(),
                })?;

        let size = ByteLength::new(metadata.len());
        let checksum_path = self.checksum_file_path(model);
        let digest = if tokio::fs::try_exists(&checksum_path).await.unwrap_or(false) {
            if let Ok(content) = tokio::fs::read_to_string(&checksum_path).await {
                Checksum::parse(content.trim()).ok()
            } else {
                None
            }
        } else {
            None
        };

        Ok(InstalledModel::new(model.clone(), path, size, digest))
    }
}

fn dirs_path() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{ModelFileName, ModelRepository, ModelRepositoryId};
    use tempfile::TempDir;

    fn test_spec() -> ModelSpec {
        let identifier = ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id");
        ModelSpec::new(
            ModelRepository::at_default_revision(identifier),
            ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file name"),
        )
    }

    #[tokio::test]
    async fn missing_model_returns_missing_state() {
        let temp_dir = TempDir::new().expect("temp dir");
        let library = DiskModelLibrary::new(temp_dir.path());
        let spec = test_spec();

        let state = library.installed_state(&spec).await.expect("state query");
        assert_eq!(state, ModelState::Missing);
    }

    #[tokio::test]
    async fn commit_artifact_places_file_in_hierarchy() {
        let temp_dir = TempDir::new().expect("temp dir");
        let library = DiskModelLibrary::new(temp_dir.path());
        let spec = test_spec();

        let staged_dir = TempDir::new().expect("staged dir");
        let staged_file = staged_dir.path().join("downloaded.bin");
        tokio::fs::write(&staged_file, b"sample model payload")
            .await
            .expect("write staged");

        let artifact = ModelArtifact::new(&staged_file, ByteLength::new(20));
        let state = library
            .commit_artifact(&spec, &artifact)
            .await
            .expect("commit");
        assert_eq!(state, ModelState::Downloaded);

        let installed_state = library
            .installed_state(&spec)
            .await
            .expect("installed state");
        assert_eq!(installed_state, ModelState::Downloaded);
    }

    #[tokio::test]
    async fn verify_integrity_with_matching_checksum_marks_verified() {
        let temp_dir = TempDir::new().expect("temp dir");
        let library = DiskModelLibrary::new(temp_dir.path());
        let spec = test_spec();

        let payload = b"correct model content bytes";
        let staged_dir = TempDir::new().expect("staged dir");
        let staged_file = staged_dir.path().join("staged.bin");
        tokio::fs::write(&staged_file, payload)
            .await
            .expect("write");

        let mut hasher = Sha256::new();
        hasher.update(payload);
        let expected_digest = Checksum::from_bytes(hasher.finalize().into());

        let artifact = ModelArtifact::new(&staged_file, ByteLength::new(payload.len() as u64));
        library
            .commit_artifact(&spec, &artifact)
            .await
            .expect("commit");

        let verify_result = library
            .verify_integrity(&spec, Some(expected_digest))
            .await
            .expect("verify");
        assert_eq!(verify_result, ModelState::Verified);

        let state_after = library.installed_state(&spec).await.expect("state");
        assert_eq!(state_after, ModelState::Verified);

        let located = library.locate(&spec).await.expect("locate");
        assert_eq!(located.digest(), Some(expected_digest));
        assert_eq!(located.size(), ByteLength::new(payload.len() as u64));
    }

    #[tokio::test]
    async fn verify_integrity_with_mismatching_checksum_reports_mismatch() {
        let temp_dir = TempDir::new().expect("temp dir");
        let library = DiskModelLibrary::new(temp_dir.path());
        let spec = test_spec();

        let payload = b"corrupted payload";
        let staged_dir = TempDir::new().expect("staged dir");
        let staged_file = staged_dir.path().join("staged.bin");
        tokio::fs::write(&staged_file, payload)
            .await
            .expect("write");

        let expected_digest = Checksum::from_bytes([0x99; 32]);
        let artifact = ModelArtifact::new(&staged_file, ByteLength::new(payload.len() as u64));
        library
            .commit_artifact(&spec, &artifact)
            .await
            .expect("commit");

        let verify_result = library
            .verify_integrity(&spec, Some(expected_digest))
            .await
            .expect("verify");

        let mut hasher = Sha256::new();
        hasher.update(payload);
        let actual_digest = Checksum::from_bytes(hasher.finalize().into());

        assert_eq!(
            verify_result,
            ModelState::IntegrityMismatch {
                expected: expected_digest,
                actual: actual_digest,
            }
        );
    }

    #[tokio::test]
    async fn verify_integrity_without_expected_digest_keeps_downloaded_state() {
        let temp_dir = TempDir::new().expect("temp dir");
        let library = DiskModelLibrary::new(temp_dir.path());
        let spec = test_spec();

        let staged_dir = TempDir::new().expect("staged dir");
        let staged_file = staged_dir.path().join("staged.bin");
        tokio::fs::write(&staged_file, b"payload")
            .await
            .expect("write");

        let artifact = ModelArtifact::new(&staged_file, ByteLength::new(7));
        library
            .commit_artifact(&spec, &artifact)
            .await
            .expect("commit");

        let verify_result = library.verify_integrity(&spec, None).await.expect("verify");
        assert_eq!(verify_result, ModelState::Downloaded);

        let located = library.locate(&spec).await.expect("locate");
        assert_eq!(located.digest(), None);
    }

    #[tokio::test]
    async fn locate_on_missing_model_returns_unreadable_error() {
        let temp_dir = TempDir::new().expect("temp dir");
        let library = DiskModelLibrary::new(temp_dir.path());
        let spec = test_spec();

        let result = library.locate(&spec).await;
        assert!(matches!(result, Err(LibraryError::Unreadable { .. })));
    }
}
