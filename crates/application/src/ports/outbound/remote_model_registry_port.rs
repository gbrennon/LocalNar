use domain::{ModelFileName, ModelRepository, RemoteModelFile, SearchQuery};

use crate::errors::registry_read_error::RegistryReadError;

/// Outbound contract for reading what an upstream catalog offers.
///
/// Implementations translate a Hugging Face Hub (or equivalent) catalog into
/// domain `RemoteModelFile` values; no layer above infrastructure performs
/// network I/O.
pub trait RemoteModelRegistryPort: Send + Sync {
    /// Resolves metadata for one requested file inside a repository.
    async fn resolve_model_file(
        &self,
        repository: &ModelRepository,
        file: &ModelFileName,
    ) -> Result<RemoteModelFile, RegistryReadError>;

    /// Finds every downloadable file matching a free-text `query`.
    ///
    /// The result is already flattened: one entry per downloadable file, not
    /// per repository, so a caller can render candidates without a second
    /// round trip. How an adapter reaches that shape - one catalog call or a
    /// fan out across repositories - is its own concern.
    ///
    /// Adapters that cannot search may return
    /// `RegistryReadError::EnumerationUnsupported`.
    async fn search_models(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<RemoteModelFile>, RegistryReadError> {
        let _ = query;
        Err(RegistryReadError::EnumerationUnsupported)
    }
}
