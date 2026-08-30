#![allow(dead_code)]
use std::sync::atomic::{AtomicBool, Ordering};

use application::errors::LibraryError;
use application::ports::outbound::ModelLibraryPort;
use domain::{Checksum, InstalledModel, ModelArtifact, ModelSpec, ModelState};

/// A library that holds a replica until something discards it.
///
/// The first reading reports the replica the operator asked about; every later
/// reading reports it gone, which is what a library looks like across a removal
/// that really happened.
pub struct FakeVacatingModelLibrary {
    vacated: AtomicBool,
}

impl FakeVacatingModelLibrary {
    /// Builds a library still holding its replica.
    pub fn new() -> Self {
        Self {
            vacated: AtomicBool::new(false),
        }
    }
}

impl ModelLibraryPort for FakeVacatingModelLibrary {
    async fn installed_state(&self, _model: &ModelSpec) -> Result<ModelState, LibraryError> {
        if self.vacated.swap(true, Ordering::SeqCst) {
            Ok(ModelState::Missing)
        } else {
            Ok(ModelState::Verified)
        }
    }

    async fn commit_artifact(
        &self,
        _model: &ModelSpec,
        _artifact: &ModelArtifact,
    ) -> Result<ModelState, LibraryError> {
        panic!("removing a model must never commit anything into the library")
    }

    async fn verify_integrity(
        &self,
        _model: &ModelSpec,
        _expected: Option<Checksum>,
    ) -> Result<ModelState, LibraryError> {
        panic!("removing a model must never spend a read proving it first")
    }

    async fn locate(&self, _model: &ModelSpec) -> Result<InstalledModel, LibraryError> {
        panic!("removing a model needs no description of where its bytes were")
    }
}
