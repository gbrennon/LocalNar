use domain::{ModelFileName, ModelInfo, ModelRepository, RemoteModelFile, SearchQuery};

use crate::errors::registry_read_error::RegistryReadError;

/// Outbound contract for reading what an upstream catalog offers.
///
/// Implementations translate a Hugging Face Hub (or equivalent) catalog into
/// domain values; no layer above infrastructure performs network I/O.
pub trait RemoteModelRegistryPort: Send + Sync {
    /// Reads the size and advertised digest of one named file.
    async fn resolve_model_file(
        &self,
        repository: &ModelRepository,
        file: &ModelFileName,
    ) -> Result<RemoteModelFile, RegistryReadError>;

    /// Describes each catalog entry matching `query` as exactly one candidate.
    ///
    /// An entry publishes many files, of which at most one stands for the model,
    /// so an implementation must answer with one `ModelInfo` per entry and never
    /// with one per published file. An entry it cannot describe as a single
    /// installable candidate is left out rather than reported partially.
    async fn search_models(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<ModelInfo>, RegistryReadError>;
}
