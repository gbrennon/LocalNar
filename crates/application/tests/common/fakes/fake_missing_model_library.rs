#![allow(dead_code)]
use application::{errors::LibraryError, ports::outbound::ModelLibraryPort};
use domain::{Checksum, InstalledModel, ModelArtifact, ModelSpec, ModelState};

use crate::common::fakes::model_fixture::ModelFixture;

/// A library that holds no replica for the requested model.
pub struct FakeMissingModelLibrary;

impl ModelLibraryPort for FakeMissingModelLibrary {
    async fn installed_state(&self, _model: &ModelSpec) -> Result<ModelState, LibraryError> {
        Ok(ModelState::Missing)
    }

    async fn commit_artifact(
        &self,
        _model: &ModelSpec,
        _artifact: &ModelArtifact,
    ) -> Result<ModelState, LibraryError> {
        Ok(ModelState::Downloaded)
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
