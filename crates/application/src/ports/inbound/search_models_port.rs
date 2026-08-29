use domain::{RemoteModelFile, SearchQuery};

use crate::errors::search_models_error::SearchModelsError;

/// Inbound contract for discovering downloadable models by free text.
///
/// The use case only reads the upstream catalog; nothing is downloaded and the
/// durable library is never consulted. Each result is one downloadable file,
/// carrying the repository, file name, and size a candidate row needs.
pub trait SearchModelsPort: Send + Sync {
    /// Lists every downloadable file matching `query`.
    async fn execute(&self, query: &SearchQuery)
    -> Result<Vec<RemoteModelFile>, SearchModelsError>;
}
