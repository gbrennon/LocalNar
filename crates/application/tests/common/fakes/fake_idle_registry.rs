#![allow(dead_code)]
use application::{errors::RegistryReadError, ports::outbound::RemoteModelRegistryPort};
use domain::{ModelFileName, ModelInfo, ModelRepository, RemoteModelFile, SearchQuery};

/// A registry that offers no enumeration and must never reach the network.
pub struct FakeIdleRegistry;

impl RemoteModelRegistryPort for FakeIdleRegistry {
    async fn resolve_model_file(
        &self,
        _repository: &ModelRepository,
        _file: &ModelFileName,
    ) -> Result<RemoteModelFile, RegistryReadError> {
        panic!("the registry must not be consulted in this scenario")
    }

    async fn search_models(
        &self,
        _query: &SearchQuery,
    ) -> Result<Vec<ModelInfo>, RegistryReadError> {
        Err(RegistryReadError::EnumerationUnsupported)
    }
}
