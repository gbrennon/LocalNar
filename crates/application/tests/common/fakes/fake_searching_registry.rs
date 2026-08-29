#![allow(dead_code)]
use application::errors::RegistryReadError;
use application::ports::outbound::RemoteModelRegistryPort;
use domain::{ModelFileName, ModelInfo, ModelRepository, RemoteModelFile, SearchQuery};

use crate::common::fakes::model_fixture::ModelFixture;

/// A registry that answers a search with one described model.
pub struct FakeSearchingRegistry;

impl RemoteModelRegistryPort for FakeSearchingRegistry {
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
        Ok(vec![ModelFixture::model_info()])
    }
}
