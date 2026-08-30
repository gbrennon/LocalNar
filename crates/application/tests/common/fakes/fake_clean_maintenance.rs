#![allow(dead_code)]
use application::errors::LibraryError;
use application::ports::outbound::LibraryMaintenancePort;
use domain::DiscardedStray;

/// Maintenance of a library that keeps nothing but its replicas.
pub struct FakeCleanMaintenance;

impl LibraryMaintenancePort for FakeCleanMaintenance {
    async fn discard_strays(&self) -> Result<Vec<DiscardedStray>, LibraryError> {
        Ok(Vec::new())
    }
}
