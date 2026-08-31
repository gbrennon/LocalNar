//! Reading the whole of what the filesystem-backed library holds.

use application::errors::LibraryError;
use application::ports::outbound::ModelInventoryPort;
use domain::ModelInventory;

use super::inventory_walk::InventoryWalk;
use super::model_library::DiskModelLibrary;

impl ModelInventoryPort for DiskModelLibrary {
    /// Describes the library's location and every replica it holds.
    ///
    /// The answer is taken by walking the hierarchy the library writes, reading
    /// the state it already recorded for each replica and nothing more. No file
    /// is read for its bytes, which is what makes the listing cheap enough to
    /// serve an operator waiting on it, and what leaves proving a replica to
    /// the caller that asks for it.
    async fn enumerate(&self) -> Result<ModelInventory, LibraryError> {
        let replicas = InventoryWalk::rooted_at(self.root()).replicas().await?;

        Ok(ModelInventory::new(self.root(), replicas))
    }
}
