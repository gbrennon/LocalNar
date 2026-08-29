use domain::{ModelFileName, ModelRepository, RemoteModelFile, SearchQuery};

use crate::errors::registry_read_error::RegistryReadError;

/// Outbound contract for reading what an upstream catalog offers.
///
/// Implementations translate a Hugging Face Hub (or equivalent) catalog into
/// domain `RemoteModelFile` values; no layer above infrastructure performs
/// network I/O.
pub trait RemoteModelRegistryPort: Send + Sync {
    async fn resolve_model_file(
        &self,
        repository: &ModelRepository,
        file: &ModelFileName,
    ) -> Result<RemoteModelFile, RegistryReadError>;

    async fn search_models(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<RemoteModelFile>, RegistryReadError>;
}
