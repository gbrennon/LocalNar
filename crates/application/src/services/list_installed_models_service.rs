use domain::ModelInventory;

use crate::errors::list_installed_models_error::ListInstalledModelsError;
use crate::ports::inbound::list_installed_models_port::ListInstalledModelsPort;
use crate::ports::outbound::model_inventory_port::ModelInventoryPort;

/// The use case that shows the operator every model this machine holds.
///
/// It depends on the inventory alone, so a listing can never install, verify, or
/// discard anything, and never reaches upstream.
pub struct ListInstalledModelsService<Inventory>
where
    Inventory: ModelInventoryPort,
{
    inventory: Inventory,
}

impl<Inventory> ListInstalledModelsService<Inventory>
where
    Inventory: ModelInventoryPort,
{
    /// Compose the use case from the inventory port.
    pub fn new(inventory: Inventory) -> Self {
        Self { inventory }
    }
}

impl<Inventory> ListInstalledModelsPort for ListInstalledModelsService<Inventory>
where
    Inventory: ModelInventoryPort,
{
    async fn execute(&self) -> Result<ModelInventory, ListInstalledModelsError> {
        Ok(self.inventory.enumerate().await?)
    }
}
