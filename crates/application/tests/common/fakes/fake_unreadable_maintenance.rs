#![allow(dead_code)]
use localnar_application::{errors::LibraryError, ports::outbound::LibraryMaintenancePort};
use localnar_domain::DiscardedStray;

/// Maintenance of a library that cannot be walked to find its leftovers.
pub struct FakeUnreadableMaintenance;

impl FakeUnreadableMaintenance {
    /// The error every sweep produces.
    pub fn error() -> LibraryError {
        LibraryError::Unreadable {
            model: "the whole library".to_string(),
            cause: "permission denied".to_string(),
        }
    }
}

impl LibraryMaintenancePort for FakeUnreadableMaintenance {
    async fn discard_strays(&self) -> Result<Vec<DiscardedStray>, LibraryError> {
        Err(Self::error())
    }
}
