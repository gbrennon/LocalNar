#![allow(dead_code)]
use application::errors::LibraryError;
use application::ports::outbound::ModelLibraryPort;
use domain::{Checksum, InstalledModel, ModelArtifact, ModelSpec, ModelState};

use crate::common::fakes::model_fixture::ModelFixture;

/// A library whose replica keeps mismatching however often it is replaced.
pub struct FakeStubbornModelLibrary;

impl FakeStubbornModelLibrary {
    fn mismatch() -> ModelState {
        ModelState::IntegrityMismatch {
            expected: ModelFixture::expected_digest(),
            actual: ModelFixture::actual_digest(),
        }
    }
}

impl ModelLibraryPort for FakeStubbornModelLibrary {
    async fn installed_state(&self, _model: &ModelSpec) -> Result<ModelState, LibraryError> {
        Ok(Self::mismatch())
    }

    async fn commit_artifact(
        &self,
        _model: &ModelSpec,
        _artifact: &ModelArtifact,
    ) -> Result<ModelState, LibraryError> {
        Ok(Self::mismatch())
    }

    async fn verify_integrity(
        &self,
        _model: &ModelSpec,
        _expected: Option<Checksum>,
    ) -> Result<ModelState, LibraryError> {
        Ok(Self::mismatch())
    }

    async fn locate(&self, _model: &ModelSpec) -> Result<InstalledModel, LibraryError> {
        panic!("a replica that never matches is never located")
    }
}
