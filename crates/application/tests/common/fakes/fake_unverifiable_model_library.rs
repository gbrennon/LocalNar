#![allow(dead_code)]
use crate::common::fakes::model_fixture::ModelFixture;
use application::errors::LibraryError;
use application::ports::outbound::ModelLibraryPort;
use domain::{Checksum, InstalledModel, ModelArtifact, ModelSpec, ModelState};

/// A library holding a replica that no checksum was ever advertised for.
///
/// Verification cannot prove or disprove such a replica, so it stays
/// `Downloaded` however often it is checked.
pub struct FakeUnverifiableModelLibrary;

impl ModelLibraryPort for FakeUnverifiableModelLibrary {
    async fn installed_state(&self, _model: &ModelSpec) -> Result<ModelState, LibraryError> {
        Ok(ModelState::Downloaded)
    }

    async fn commit_artifact(
        &self,
        _model: &ModelSpec,
        _artifact: &ModelArtifact,
    ) -> Result<ModelState, LibraryError> {
        panic!("a present replica must not be committed again")
    }

    async fn verify_integrity(
        &self,
        _model: &ModelSpec,
        _expected: Option<Checksum>,
    ) -> Result<ModelState, LibraryError> {
        Ok(ModelState::Downloaded)
    }

    async fn locate(&self, _model: &ModelSpec) -> Result<InstalledModel, LibraryError> {
        Ok(ModelFixture::installed(None))
    }
}
