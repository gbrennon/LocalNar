#![allow(dead_code)]
use localnar_application::{errors::LibraryError, ports::outbound::ModelEvictionPort};
use localnar_domain::{ModelSpec, RemovedModel};

use crate::common::fakes::model_fixture::ModelFixture;

/// A library that discards the replica it is asked to and reports the space.
pub struct FakeEvictingLibrary;

impl ModelEvictionPort for FakeEvictingLibrary {
    async fn evict(&self, _model: &ModelSpec) -> Result<RemovedModel, LibraryError> {
        Ok(ModelFixture::removed())
    }
}
