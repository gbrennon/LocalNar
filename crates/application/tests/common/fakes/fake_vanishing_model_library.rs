#![allow(dead_code)]
use localnar_application::{errors::LibraryError, ports::outbound::ModelLibraryPort};
use localnar_domain::{Checksum, InstalledModel, ModelArtifact, ModelSpec, ModelState};

use crate::common::fakes::model_fixture::ModelFixture;

/// A library whose replica is gone by the time its bytes are read.
///
/// It stands for a model removed by other means between the reading that placed
/// it and the reading that would have proven it.
pub struct FakeVanishingModelLibrary;

impl ModelLibraryPort for FakeVanishingModelLibrary {
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
        Ok(ModelState::Missing)
    }

    async fn locate(&self, _model: &ModelSpec) -> Result<InstalledModel, LibraryError> {
        Ok(ModelFixture::installed(Some(
            ModelFixture::expected_digest(),
        )))
    }
}
