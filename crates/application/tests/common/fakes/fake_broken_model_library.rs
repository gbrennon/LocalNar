#![allow(dead_code)]
use application::errors::LibraryError;
use application::ports::outbound::ModelLibraryPort;
use domain::{Checksum, InstalledModel, ModelArtifact, ModelSpec, ModelState};

use crate::common::fakes::model_fixture::ModelFixture;

/// A library holding a present replica whose bytes no longer hash as recorded.
///
/// The replica is reported as installed and locatable, since it is: the
/// disagreement only surfaces once its bytes are read again.
pub struct FakeBrokenModelLibrary;

impl ModelLibraryPort for FakeBrokenModelLibrary {
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
        Ok(ModelFixture::mismatched_state())
    }

    async fn locate(&self, _model: &ModelSpec) -> Result<InstalledModel, LibraryError> {
        Ok(ModelFixture::installed(Some(
            ModelFixture::expected_digest(),
        )))
    }
}
