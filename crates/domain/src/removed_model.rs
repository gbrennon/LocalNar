use std::path::{Path, PathBuf};

use crate::ByteLength;
use crate::ModelSpec;

/// The record of a replica the operator discarded from the local library.
///
/// A removal is worth reporting rather than merely succeeding: the operator
/// discards a model to reclaim the machine, so the answer names what left and
/// how much space came back. The path is the place the bytes used to occupy,
/// which no longer holds them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemovedModel {
    spec: ModelSpec,
    path: PathBuf,
    reclaimed: ByteLength,
}

impl RemovedModel {
    /// Records that the replica of `spec` at `path` gave back `reclaimed` bytes.
    pub fn new(spec: ModelSpec, path: impl Into<PathBuf>, reclaimed: ByteLength) -> Self {
        Self {
            spec,
            path: path.into(),
            reclaimed,
        }
    }

    /// The model that no longer has a replica in the library.
    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }

    /// The place the discarded bytes used to occupy.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// How much space the removal gave back.
    pub fn reclaimed(&self) -> ByteLength {
        self.reclaimed
    }
}

#[cfg(test)]
mod removed_model_tests {
    use super::RemovedModel;
    use crate::ByteLength;
    use crate::ModelFileName;
    use crate::ModelRepository;
    use crate::ModelRepositoryId;
    use crate::ModelSpec;

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
    fn a_removal_names_the_model_and_the_space_it_gave_back() {
        let removal = RemovedModel::new(spec(), "/models/qwen.gguf", ByteLength::new(4_096));

        assert_eq!(removal.spec(), &spec());
        assert_eq!(removal.path().to_str(), Some("/models/qwen.gguf"));
        assert_eq!(removal.reclaimed(), ByteLength::new(4_096));
    }
}
