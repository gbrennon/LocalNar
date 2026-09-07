use localnar_domain::{ModelFileName, ModelInfo, ModelRepository, RemoteModelFile, SearchQuery};

use crate::errors::registry_read_error::RegistryReadError;

pub trait RemoteModelRegistryPort: Send + Sync {
    async fn resolve_model_file(
        &self,
        repository: &ModelRepository,
        file: &ModelFileName,
    ) -> Result<RemoteModelFile, RegistryReadError>;

    async fn search_models(&self, query: &SearchQuery)
    -> Result<Vec<ModelInfo>, RegistryReadError>;
}
