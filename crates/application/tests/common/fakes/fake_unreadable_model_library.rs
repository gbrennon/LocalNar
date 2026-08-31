#![allow(dead_code)]
use application::{errors::LibraryError, ports::outbound::ModelLibraryPort};
use domain::{Checksum, InstalledModel, ModelArtifact, ModelSpec, ModelState};

/// A library whose location cannot be read at all.
pub struct FakeUnreadableModelLibrary;

impl FakeUnreadableModelLibrary {
    /// The error every read produces.
    pub fn error() -> LibraryError {
        LibraryError::Unreadable {
            model: "qwen3-8b".to_string(),
            cause: "permission denied".to_string(),
        }
    }
}

impl ModelLibraryPort for FakeUnreadableModelLibrary {
    async fn installed_state(&self, _model: &ModelSpec) -> Result<ModelState, LibraryError> {
        Err(Self::error())
    }

    async fn commit_artifact(
        &self,
        _model: &ModelSpec,
        _artifact: &ModelArtifact,
    ) -> Result<ModelState, LibraryError> {
        Err(Self::error())
    }

    async fn verify_integrity(
        &self,
        _model: &ModelSpec,
        _expected: Option<Checksum>,
    ) -> Result<ModelState, LibraryError> {
        Err(Self::error())
    }

    async fn locate(&self, _model: &ModelSpec) -> Result<InstalledModel, LibraryError> {
        Err(Self::error())
    }
}
