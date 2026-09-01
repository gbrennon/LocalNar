#![allow(dead_code)]
use localnar_application::{errors::LibraryError, ports::outbound::ModelLibraryPort};
use localnar_domain::{Checksum, InstalledModel, ModelArtifact, ModelSpec, ModelState};

use crate::common::fakes::model_fixture::ModelFixture;

/// A library holding a replica that no digest was ever recorded for.
///
/// Hashing such a replica would read the whole file for a verdict nothing can
/// be compared against, so this library refuses to be asked.
pub struct FakeUnprovenModelLibrary;

impl ModelLibraryPort for FakeUnprovenModelLibrary {
    async fn installed_state(&self, _model: &ModelSpec) -> Result<ModelState, LibraryError> {
        Ok(ModelState::Downloaded)
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
        panic!("a replica carrying no recorded digest must never be re-read")
    }

    async fn locate(&self, _model: &ModelSpec) -> Result<InstalledModel, LibraryError> {
        Ok(ModelFixture::installed(None))
    }
}
