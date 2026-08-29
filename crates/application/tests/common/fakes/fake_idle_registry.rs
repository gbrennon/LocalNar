#![allow(dead_code)]
use application::errors::RegistryReadError;
use application::ports::outbound::RemoteModelRegistryPort;
use domain::{ModelFileName, ModelRepository, RemoteModelFile};

/// A registry for scenarios that must never reach the network.
pub struct FakeIdleRegistry;

impl RemoteModelRegistryPort for FakeIdleRegistry {
    async fn resolve_model_file(
        &self,
        _repository: &ModelRepository,
        _file: &ModelFileName,
    ) -> Result<RemoteModelFile, RegistryReadError> {
        panic!("the registry must not be consulted in this scenario")
    }
}
