use domain::{ModelInfo, SearchQuery};

use crate::{
    errors::search_models_error::SearchModelsError,
    ports::{
        inbound::search_models_port::SearchModelsPort,
        outbound::remote_model_registry_port::RemoteModelRegistryPort,
    },
};

/// The use case that finds downloadable models by free text.
///
/// It depends on the registry alone, so a search can never touch the durable
/// library or stage any bytes.
pub struct SearchModelsService<Registry>
where
    Registry: RemoteModelRegistryPort,
{
    registry: Registry,
}

impl<Registry> SearchModelsService<Registry>
where
    Registry: RemoteModelRegistryPort,
{
    /// Compose the use case from the registry port.
    pub fn new(registry: Registry) -> Self {
        Self { registry }
    }
}

impl<Registry> SearchModelsPort for SearchModelsService<Registry>
where
    Registry: RemoteModelRegistryPort,
{
    async fn execute(&self, query: &SearchQuery) -> Result<Vec<ModelInfo>, SearchModelsError> {
        Ok(self.registry.search_models(query).await?)
    }
}
