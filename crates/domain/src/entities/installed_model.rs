use std::path::{Path, PathBuf};

use crate::value_objects::{ByteLength, Checksum, ModelSpec};

/// A model replica that is present in the durable library.
///
/// This is the answer to "where did the file land": it names the model, the
/// place its bytes occupy, and how large they are. The digest is optional
/// because integrity can only be proven when upstream advertised a checksum
/// to compare against; a replica without one is installed but unproven.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledModel {
    spec: ModelSpec,
    path: PathBuf,
    size: ByteLength,
    digest: Option<Checksum>,
}

impl InstalledModel {
    /// Describes a replica the library holds for `spec`.
    pub fn new(
        spec: ModelSpec,
        path: impl Into<PathBuf>,
        size: ByteLength,
        digest: Option<Checksum>,
    ) -> Self {
        Self {
            spec,
            path: path.into(),
            size,
            digest,
        }
    }

    /// The model this replica satisfies.
    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }

    /// Where the bytes live.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// How much space the replica occupies.
    pub fn size(&self) -> ByteLength {
        self.size
    }

    /// The digest the replica was proven against, when integrity was checked.
    pub fn digest(&self) -> Option<Checksum> {
        self.digest
    }

    /// Whether the replica's integrity was proven against an advertised digest.
    pub fn is_verified(&self) -> bool {
        self.digest.is_some()
    }
}

#[cfg(test)]
mod installed_model_tests {
    use super::InstalledModel;
    use crate::value_objects::{
        ByteLength, Checksum, ModelFileName, ModelRepository, ModelRepositoryId, ModelSpec,
    };

    fn spec() -> ModelSpec {
        ModelSpec::new(
            ModelRepository::at_default_revision(
                ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id"),
            ),
            ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file name"),
            vec![],
        )
    }

    #[test]
    fn a_replica_without_a_digest_is_installed_but_unverified() {
        let replica =
            InstalledModel::new(spec(), "/models/qwen.gguf", ByteLength::new(4_096), None);

        assert!(!replica.is_verified());
        assert_eq!(replica.path().to_str(), Some("/models/qwen.gguf"));
        assert_eq!(replica.size(), ByteLength::new(4_096));
    }

    #[test]
    fn a_replica_carrying_a_digest_is_verified() {
        let digest = Checksum::from_bytes([0x11; 32]);
        let replica = InstalledModel::new(
            spec(),
            "/models/qwen.gguf",
            ByteLength::new(4_096),
            Some(digest),
        );

        assert!(replica.is_verified());
        assert_eq!(replica.digest(), Some(digest));
        assert_eq!(replica.spec(), &spec());
    }
}
