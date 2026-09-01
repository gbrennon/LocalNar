#![allow(dead_code)]
use localnar_application::{errors::LibraryError, ports::outbound::ModelLibraryPort};
use localnar_domain::{Checksum, InstalledModel, ModelArtifact, ModelSpec, ModelState};

use crate::common::fakes::model_fixture::ModelFixture;

/// A library holding a replica that is already complete and verified.
pub struct FakeVerifiedModelLibrary;

impl ModelLibraryPort for FakeVerifiedModelLibrary {
    async fn installed_state(&self, _model: &ModelSpec) -> Result<ModelState, LibraryError> {
        Ok(ModelState::Verified)
    }

    async fn commit_artifact(
        &self,
        _model: &ModelSpec,
        _artifact: &ModelArtifact,
    ) -> Result<ModelState, LibraryError> {
        panic!("a verified replica must not be committed again")
    }

    async fn verify_integrity(
        &self,
        _model: &ModelSpec,
        _expected: Option<Checksum>,
    ) -> Result<ModelState, LibraryError> {
        Ok(ModelState::Verified)
    }

    async fn locate(&self, _model: &ModelSpec) -> Result<InstalledModel, LibraryError> {
        Ok(ModelFixture::installed(Some(
            ModelFixture::expected_digest(),
        )))
    }
}
