#![allow(dead_code)]
use localnar_application::{errors::RegistryReadError, ports::outbound::RemoteModelRegistryPort};
use localnar_domain::{ModelFileName, ModelInfo, ModelRepository, RemoteModelFile, SearchQuery};

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
    ) -> Result<Vec<ModelInfo>, RegistryReadError> {
        panic!("an install scenario must not search the catalog")
    }
}
