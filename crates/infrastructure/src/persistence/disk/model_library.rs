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
        let path = std::env::var("LOCALNAR_MODELS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| Self::default_root());
        Self::new(path)
    }

    /// Returns the root directory path where models are stored.
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn default_root() -> PathBuf {
        dirs_path().join(".cache").join("localnar").join("models")
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

    /// Frees `path` of whatever entry occupies it, so the library alone decides
    /// what lives there.
    ///
    /// An entry left by an earlier install is discarded rather than written
    /// through. A symlink is the case that matters: downloaders stage their
    /// files as links into their own cache, so an install that once moved such
    /// a link into the library leaves a relative target that no longer resolves
    /// from here. Writing through that link would either fail against the
    /// vanished target or push the model bytes outside the library and overwrite
    /// whatever the link pointed at, and in both cases the model stays
    /// unreachable at the path the library reads. An absent path is already
    /// vacant and reports success.
    async fn vacate(path: &Path, model: &ModelSpec) -> Result<(), LibraryError> {
        let occupant = match tokio::fs::symlink_metadata(path).await {
            Ok(occupant) => occupant,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => {
                return Err(LibraryError::Unwritable {
                    model: model.to_string(),
                    cause: err.to_string(),
                });
            }
        };

        let removal = if occupant.is_dir() {
            tokio::fs::remove_dir_all(path).await
        } else {
            tokio::fs::remove_file(path).await
        };

        removal.map_err(|err| LibraryError::Unwritable {
            model: model.to_string(),
            cause: err.to_string(),
        })
    }

    /// Proves that `path` now holds a readable file of its own.
    ///
    /// A commit that answers `Downloaded` promises the bytes are installed, and
    /// callers act on that promise by asking the library to verify or serve
    /// them. Reporting the promise without evidence turns a storage fault into a
    /// verdict about upstream, so a destination that resolves to nothing is
    /// reported as the write failure it is.
    async fn confirm_committed(path: &Path, model: &ModelSpec) -> Result<(), LibraryError> {
        let committed =
            tokio::fs::metadata(path)
                .await
                .map_err(|err| LibraryError::Unwritable {
                    model: model.to_string(),
                    cause: err.to_string(),
                })?;

        if committed.is_file() {
            return Ok(());
        }

        Err(LibraryError::Unwritable {
            model: model.to_string(),
            cause: format!("`{}` is not a regular file", path.display()),
        })
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
        if staged != destination {
            Self::vacate(&destination, model).await?;
            tokio::fs::copy(staged, &destination).await.map_err(|err| {
                LibraryError::Unwritable {
                    model: model.to_string(),
                    cause: err.to_string(),
                }
            })?;
            let _ = tokio::fs::remove_file(staged).await;
        }

        Self::confirm_committed(&destination, model).await?;

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

    /// Regression: a downloader (hf-hub) stages its artifact as a symlink into
    /// its own cache. Moving that link into the library would leave a dangling
    /// reference whose relative target no longer resolves, making the committed
    /// model read as `Missing`. Commit must materialize real bytes instead.
    #[tokio::test]
    async fn commit_of_symlink_artifact_materializes_real_file() {
        let temp_dir = TempDir::new().expect("temp dir");
        let library = DiskModelLibrary::new(temp_dir.path());
        let spec = test_spec();

        let staged_dir = TempDir::new().expect("staged dir");
        // A relative symlink, like hf-hub creates between its snapshot and blob.
        let blob = staged_dir.path().join("blob.bin");
        tokio::fs::write(&blob, b"hf model weights bytes")
            .await
            .expect("write blob");
        let staged_symlink = staged_dir.path().join("downloaded.gguf");
        std::os::unix::fs::symlink("blob.bin", &staged_symlink).expect("symlink");

        let artifact = ModelArtifact::new(
            &staged_symlink,
            ByteLength::new(b"hf model weights bytes".len() as u64),
        );
        let state = library
            .commit_artifact(&spec, &artifact)
            .await
            .expect("commit");
        assert_eq!(state, ModelState::Downloaded);

        // The committed model must be discoverable (not a dangling link).
        let installed_state = library
            .installed_state(&spec)
            .await
            .expect("installed state");
        assert_eq!(installed_state, ModelState::Downloaded);

        // And must hold the real bytes under the library root.
        let located = library.locate(&spec).await.expect("locate");
        let bytes = tokio::fs::read(located.path()).await.expect("read model");
        assert_eq!(bytes, b"hf model weights bytes");
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

    /// Regression: an install interrupted by an earlier defect can leave a
    /// symlink at the destination whose relative target does not resolve from
    /// the library. A later commit must take the destination over and install
    /// real bytes rather than fail against the stale link.
    #[tokio::test]
    async fn commit_over_a_dangling_destination_symlink_installs_real_bytes() {
        let temp_dir = TempDir::new().expect("temp dir");
        let library = DiskModelLibrary::new(temp_dir.path());
        let spec = test_spec();

        let destination = library.model_file_path(&spec);
        tokio::fs::create_dir_all(destination.parent().expect("parent"))
            .await
            .expect("create parent");
        std::os::unix::fs::symlink("../../blobs/vanished", &destination).expect("stale symlink");

        let staged_dir = TempDir::new().expect("staged dir");
        let staged_file = staged_dir.path().join("downloaded.gguf");
        tokio::fs::write(&staged_file, b"freshly downloaded weights")
            .await
            .expect("write staged");

        let artifact = ModelArtifact::new(
            &staged_file,
            ByteLength::new(b"freshly downloaded weights".len() as u64),
        );

        let state = library
            .commit_artifact(&spec, &artifact)
            .await
            .expect("commit over stale link");
        assert_eq!(state, ModelState::Downloaded);

        let installed_state = library
            .installed_state(&spec)
            .await
            .expect("installed state");
        assert_eq!(installed_state, ModelState::Downloaded);

        let bytes = tokio::fs::read(&destination).await.expect("read model");
        assert_eq!(bytes, b"freshly downloaded weights");
    }

    /// Regression: a symlink at the destination must never redirect the commit.
    /// Following it would write the model bytes outside the library root and
    /// clobber whatever the link happened to point at.
    #[tokio::test]
    async fn commit_never_writes_through_a_destination_symlink() {
        let temp_dir = TempDir::new().expect("temp dir");
        let library = DiskModelLibrary::new(temp_dir.path());
        let spec = test_spec();

        let outside_dir = TempDir::new().expect("outside dir");
        let bystander = outside_dir.path().join("unrelated.bin");
        tokio::fs::write(&bystander, b"unrelated file contents")
            .await
            .expect("write bystander");

        let destination = library.model_file_path(&spec);
        tokio::fs::create_dir_all(destination.parent().expect("parent"))
            .await
            .expect("create parent");
        std::os::unix::fs::symlink(&bystander, &destination).expect("escaping symlink");

        let staged_dir = TempDir::new().expect("staged dir");
        let staged_file = staged_dir.path().join("downloaded.gguf");
        tokio::fs::write(&staged_file, b"model weights")
            .await
            .expect("write staged");

        let artifact = ModelArtifact::new(&staged_file, ByteLength::new(13));
        library
            .commit_artifact(&spec, &artifact)
            .await
            .expect("commit over escaping link");

        let bystander_bytes = tokio::fs::read(&bystander).await.expect("read bystander");
        assert_eq!(bystander_bytes, b"unrelated file contents");

        let installed_bytes = tokio::fs::read(&destination).await.expect("read model");
        assert_eq!(installed_bytes, b"model weights");
    }

    /// Regression: reporting `Downloaded` for a destination that holds nothing
    /// makes the caller blame upstream for bytes the library lost.
    #[tokio::test]
    async fn commit_of_an_artifact_already_at_the_destination_keeps_it() {
        let temp_dir = TempDir::new().expect("temp dir");
        let library = DiskModelLibrary::new(temp_dir.path());
        let spec = test_spec();

        let destination = library.model_file_path(&spec);
        tokio::fs::create_dir_all(destination.parent().expect("parent"))
            .await
            .expect("create parent");
        tokio::fs::write(&destination, b"already in place")
            .await
            .expect("write destination");

        let artifact = ModelArtifact::new(&destination, ByteLength::new(16));
        let state = library
            .commit_artifact(&spec, &artifact)
            .await
            .expect("commit in place");
        assert_eq!(state, ModelState::Downloaded);

        let bytes = tokio::fs::read(&destination).await.expect("read model");
        assert_eq!(bytes, b"already in place");
    }

    #[tokio::test]
    async fn commit_of_an_absent_artifact_at_the_destination_reports_unwritable() {
        let temp_dir = TempDir::new().expect("temp dir");
        let library = DiskModelLibrary::new(temp_dir.path());
        let spec = test_spec();

        let destination = library.model_file_path(&spec);
        let artifact = ModelArtifact::new(&destination, ByteLength::new(16));

        let result = library.commit_artifact(&spec, &artifact).await;
        assert!(matches!(result, Err(LibraryError::Unwritable { .. })));

        let state = library.installed_state(&spec).await.expect("state");
        assert_eq!(state, ModelState::Missing);
    }

    #[tokio::test]
    async fn commit_of_an_absent_staged_artifact_reports_unwritable() {
        let temp_dir = TempDir::new().expect("temp dir");
        let library = DiskModelLibrary::new(temp_dir.path());
        let spec = test_spec();

        let staged_dir = TempDir::new().expect("staged dir");
        let artifact = ModelArtifact::new(
            staged_dir.path().join("never-downloaded.gguf"),
            ByteLength::new(10),
        );

        let result = library.commit_artifact(&spec, &artifact).await;
        assert!(matches!(result, Err(LibraryError::Unwritable { .. })));

        let state = library.installed_state(&spec).await.expect("state");
        assert_eq!(state, ModelState::Missing);
    }

    #[tokio::test]
    async fn commit_replaces_an_already_verified_model_and_clears_its_digest() {
        let temp_dir = TempDir::new().expect("temp dir");
        let library = DiskModelLibrary::new(temp_dir.path());
        let spec = test_spec();

        let first_payload = b"first installed payload";
        let staged_dir = TempDir::new().expect("staged dir");
        let first_staged = staged_dir.path().join("first.bin");
        tokio::fs::write(&first_staged, first_payload)
            .await
            .expect("write first");

        let mut hasher = Sha256::new();
        hasher.update(first_payload);
        let first_digest = Checksum::from_bytes(hasher.finalize().into());

        library
            .commit_artifact(
                &spec,
                &ModelArtifact::new(&first_staged, ByteLength::new(first_payload.len() as u64)),
            )
            .await
            .expect("first commit");
        library
            .verify_integrity(&spec, Some(first_digest))
            .await
            .expect("first verify");
        assert_eq!(
            library.installed_state(&spec).await.expect("state"),
            ModelState::Verified
        );

        let second_payload = b"second installed payload";
        let second_staged = staged_dir.path().join("second.bin");
        tokio::fs::write(&second_staged, second_payload)
            .await
            .expect("write second");

        let state = library
            .commit_artifact(
                &spec,
                &ModelArtifact::new(&second_staged, ByteLength::new(second_payload.len() as u64)),
            )
            .await
            .expect("second commit");
        assert_eq!(state, ModelState::Downloaded);
        assert_eq!(
            library.installed_state(&spec).await.expect("state"),
            ModelState::Downloaded
        );

        let bytes = tokio::fs::read(library.model_file_path(&spec))
            .await
            .expect("read model");
        assert_eq!(bytes, second_payload);

        let located = library.locate(&spec).await.expect("locate");
        assert_eq!(located.digest(), None);
    }
}
