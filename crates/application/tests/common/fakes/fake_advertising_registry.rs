#![allow(dead_code)]
use application::errors::RegistryReadError;
use application::ports::outbound::RemoteModelRegistryPort;
use domain::{ModelFileName, ModelRepository, RemoteModelFile, SearchQuery};

use crate::common::fakes::model_fixture::ModelFixture;

/// A registry that publishes the fixture file with its advertised digest.
pub struct FakeAdvertisingRegistry;

impl RemoteModelRegistryPort for FakeAdvertisingRegistry {
    async fn resolve_model_file(
        &self,
        _repository: &ModelRepository,
        _file: &ModelFileName,
    ) -> Result<RemoteModelFile, RegistryReadError> {
        Ok(ModelFixture::remote_file())
    }

    async fn search_models(
        &self,
        _query: &SearchQuery,
    ) -> Result<Vec<RemoteModelFile>, RegistryReadError> {
        panic!("an install scenario must not search the catalog")
    }
}
