//! Clearing what the filesystem-backed library keeps that is not a model.

use application::{errors::LibraryError, ports::outbound::LibraryMaintenancePort};
use domain::DiscardedStray;

use super::{library_sweep::LibrarySweep, model_library::DiskModelLibrary};

impl LibraryMaintenancePort for DiskModelLibrary {
    /// Discards every leftover the library keeps that stands for no model.
    ///
    /// The sweep reaches only digest notes whose replica is gone and
    /// directories left holding nothing, so no model the operator installed is
    /// made unreachable by it and no file the library did not put there is
    /// destroyed. A sweep that found nothing to discard answers with nothing
    /// discarded.
    async fn discard_strays(&self) -> Result<Vec<DiscardedStray>, LibraryError> {
        LibrarySweep::rooted_at(self.root())
            .discarded_strays()
            .await
    }
}
