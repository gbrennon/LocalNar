#![allow(dead_code)]
use localnar_application::{errors::LibraryError, ports::outbound::LibraryMaintenancePort};
use localnar_domain::DiscardedStray;

/// Maintenance of a library that keeps nothing but its replicas.
pub struct FakeCleanMaintenance;

impl LibraryMaintenancePort for FakeCleanMaintenance {
    async fn discard_strays(&self) -> Result<Vec<DiscardedStray>, LibraryError> {
        Ok(Vec::new())
    }
}
