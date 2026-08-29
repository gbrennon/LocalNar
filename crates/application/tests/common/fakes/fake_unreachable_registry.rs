#![allow(dead_code)]
use application::errors::RegistryReadError;
use application::ports::outbound::RemoteModelRegistryPort;
use domain::{ModelFileName, ModelRepository, RemoteModelFile, SearchQuery};

/// A registry whose host never answers.
pub struct FakeUnreachableRegistry;

impl FakeUnreachableRegistry {
    /// The error every resolution attempt produces.
    pub fn error() -> RegistryReadError {
        RegistryReadError::Unreachable {
            repository: "unsloth/Qwen3-8B-GGUF".to_string(),
            cause: "connection refused".to_string(),
        }
    }
}

impl RemoteModelRegistryPort for FakeUnreachableRegistry {
    async fn resolve_model_file(
        &self,
        _repository: &ModelRepository,
        _file: &ModelFileName,
    ) -> Result<RemoteModelFile, RegistryReadError> {
        Err(Self::error())
    }

    async fn search_models(
        &self,
        _query: &SearchQuery,
    ) -> Result<Vec<RemoteModelFile>, RegistryReadError> {
        Err(Self::error())
    }
}
