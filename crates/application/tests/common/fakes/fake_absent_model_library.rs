#![allow(dead_code)]
use application::{errors::LibraryError, ports::outbound::ModelLibraryPort};
use domain::{Checksum, InstalledModel, ModelArtifact, ModelSpec, ModelState};

/// A library that stays empty however much is committed into it.
///
/// It stands for an upstream that answers a download without ever supplying
/// the bytes, which must not be mistaken for a completed install.
pub struct FakeAbsentModelLibrary;

impl ModelLibraryPort for FakeAbsentModelLibrary {
    async fn installed_state(&self, _model: &ModelSpec) -> Result<ModelState, LibraryError> {
        Ok(ModelState::Missing)
    }

    async fn commit_artifact(
        &self,
        _model: &ModelSpec,
        _artifact: &ModelArtifact,
    ) -> Result<ModelState, LibraryError> {
        Ok(ModelState::Missing)
    }

    async fn verify_integrity(
        &self,
        _model: &ModelSpec,
        _expected: Option<Checksum>,
    ) -> Result<ModelState, LibraryError> {
        panic!("an absent replica has nothing to verify")
    }

    async fn locate(&self, _model: &ModelSpec) -> Result<InstalledModel, LibraryError> {
        panic!("an absent replica has no location")
    }
}
