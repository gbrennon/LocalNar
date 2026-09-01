#![allow(dead_code)]
use localnar_application::{errors::LibraryError, ports::outbound::ModelInventoryPort};
use localnar_domain::ModelInventory;

/// An inventory whose library location cannot be read at all.
pub struct FakeUnreadableModelInventory;

impl FakeUnreadableModelInventory {
    /// The error every enumeration produces.
    pub fn error() -> LibraryError {
        LibraryError::Unreadable {
            model: "the whole library".to_string(),
            cause: "permission denied".to_string(),
        }
    }
}

impl ModelInventoryPort for FakeUnreadableModelInventory {
    async fn enumerate(&self) -> Result<ModelInventory, LibraryError> {
        Err(Self::error())
    }
}
