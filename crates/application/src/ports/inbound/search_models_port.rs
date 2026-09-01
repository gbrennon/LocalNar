use localnar_domain::{ModelInfo, SearchQuery};

use crate::errors::search_models_error::SearchModelsError;

/// Inbound contract for discovering downloadable models by free text.
///
/// The use case only reads the upstream catalog; nothing is downloaded and the
/// durable library is never consulted. Each result describes one catalog entry
/// as a single candidate, so the count of results is the count of models found,
/// not the count of files those models are published alongside.
pub trait SearchModelsPort: Send + Sync {
    /// Describes every model matching `query`.
    async fn execute(&self, query: &SearchQuery) -> Result<Vec<ModelInfo>, SearchModelsError>;
}
