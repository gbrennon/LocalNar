#![allow(dead_code)]
use application::errors::LibraryError;
use application::ports::outbound::ModelLibraryPort;
use domain::{Checksum, InstalledModel, ModelArtifact, ModelSpec, ModelState};

use crate::common::fakes::model_fixture::ModelFixture;

/// A library holding a replica whose bytes cannot be hashed.
///
/// The replica is present and locatable, so nothing is missing; only the
/// reading that would settle its integrity fails.
pub struct FakeUnhashableModelLibrary;

impl FakeUnhashableModelLibrary {
    /// The error every hashing attempt produces.
    pub fn error() -> LibraryError {
        LibraryError::Unverifiable {
            model: "qwen3-8b".to_string(),
            cause: "input/output error".to_string(),
        }
    }
}

impl ModelLibraryPort for FakeUnhashableModelLibrary {
    async fn installed_state(&self, _model: &ModelSpec) -> Result<ModelState, LibraryError> {
        Ok(ModelState::Verified)
    }

    async fn commit_artifact(
        &self,
        _model: &ModelSpec,
        _artifact: &ModelArtifact,
    ) -> Result<ModelState, LibraryError> {
        panic!("verifying a replica must never write to the library")
    }

    async fn verify_integrity(
        &self,
        _model: &ModelSpec,
        _expected: Option<Checksum>,
    ) -> Result<ModelState, LibraryError> {
        Err(Self::error())
    }

    async fn locate(&self, _model: &ModelSpec) -> Result<InstalledModel, LibraryError> {
        Ok(ModelFixture::installed(Some(
            ModelFixture::expected_digest(),
        )))
    }
}
