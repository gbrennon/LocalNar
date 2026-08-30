use domain::DiscardedStray;

use crate::errors::prune_library_error::PruneLibraryError;
use crate::ports::inbound::prune_library_port::PruneLibraryPort;
use crate::ports::outbound::library_maintenance_port::LibraryMaintenancePort;

/// The use case that clears what the library keeps that is not a model.
///
/// It depends on maintenance alone and holds no policy of its own: which entries
/// stand for no model is a property of how the library lays itself out, so the
/// adapter that owns the layout decides, and no installed model can be reached
/// through this use case at all.
pub struct PruneLibraryService<Maintenance>
where
    Maintenance: LibraryMaintenancePort,
{
    maintenance: Maintenance,
}

impl<Maintenance> PruneLibraryService<Maintenance>
where
    Maintenance: LibraryMaintenancePort,
{
    /// Compose the use case from the maintenance port.
    pub fn new(maintenance: Maintenance) -> Self {
        Self { maintenance }
    }
}

impl<Maintenance> PruneLibraryPort for PruneLibraryService<Maintenance>
where
    Maintenance: LibraryMaintenancePort,
{
    async fn execute(&self) -> Result<Vec<DiscardedStray>, PruneLibraryError> {
        Ok(self.maintenance.discard_strays().await?)
    }
}
