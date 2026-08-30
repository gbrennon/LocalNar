use std::sync::Arc;

use application::ports::inbound::{
    InspectModelPort, ListInstalledModelsPort, PruneLibraryPort, RemoveModelPort, VerifyModelPort,
};
use application::services::{
    InspectModelService, ListInstalledModelsService, PruneLibraryService, RemoveModelService,
    VerifyModelService,
};
use domain::ModelSpec;
use infrastructure::DiskModelLibrary;
use tokio::sync::mpsc;

use crate::tui::app_event::AppEvent;

/// Drives the model manager use cases on behalf of the interface.
///
/// Every operation an operator can perform on the models this machine already
/// holds is started here and answered as an `AppEvent`, so the interface never
/// awaits a filesystem walk or a full re-hash while it should be drawing. The
/// use cases are composed once from the durable library, which is why this owns
/// them rather than the screen that happens to show their results.
///
/// A dropped receiver means the interface is gone; a report that cannot be
/// delivered is discarded rather than failing the operation that produced it.
#[derive(Clone)]
pub struct LibraryManager {
    listing: Arc<ListInstalledModelsService<DiskModelLibrary>>,
    inspection: Arc<InspectModelService<DiskModelLibrary>>,
    verification: Arc<VerifyModelService<DiskModelLibrary>>,
    removal: Arc<RemoveModelService<DiskModelLibrary, DiskModelLibrary>>,
    pruning: Arc<PruneLibraryService<DiskModelLibrary>>,
    events: mpsc::UnboundedSender<AppEvent>,
}

impl LibraryManager {
    /// Composes every manager use case from `library`, reporting to `events`.
    pub fn new(library: DiskModelLibrary, events: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self {
            listing: Arc::new(ListInstalledModelsService::new(library.clone())),
            inspection: Arc::new(InspectModelService::new(library.clone())),
            verification: Arc::new(VerifyModelService::new(library.clone())),
            removal: Arc::new(RemoveModelService::new(library.clone(), library.clone())),
            pruning: Arc::new(PruneLibraryService::new(library)),
            events,
        }
    }

    /// Starts reading what the library holds.
    pub fn list(&self) {
        let listing = self.listing.clone();
        let events = self.events.clone();

        tokio::spawn(async move {
            let report = match listing.execute().await {
                Ok(inventory) => AppEvent::LibraryListed(inventory),
                Err(failure) => AppEvent::LibraryListingFailed(failure.to_string()),
            };
            let _ = events.send(report);
        });
    }

    /// Starts describing the replica the library holds for `spec`.
    pub fn inspect(&self, spec: ModelSpec) {
        let inspection = self.inspection.clone();
        let events = self.events.clone();

        tokio::spawn(async move {
            let report = match inspection.execute(&spec).await {
                Ok(entry) => AppEvent::ModelInspected(entry),
                Err(failure) => AppEvent::ModelInspectionFailed(failure.to_string()),
            };
            let _ = events.send(report);
        });
    }

    /// Starts re-reading the replica of `spec` to prove its recorded digest.
    pub fn verify(&self, spec: ModelSpec) {
        let verification = self.verification.clone();
        let events = self.events.clone();

        tokio::spawn(async move {
            let report = match verification.execute(&spec).await {
                Ok(entry) => AppEvent::ModelVerified(entry),
                Err(failure) => AppEvent::ModelVerificationFailed(failure.to_string()),
            };
            let _ = events.send(report);
        });
    }

    /// Starts discarding everything the library holds for `spec`.
    pub fn remove(&self, spec: ModelSpec) {
        let removal = self.removal.clone();
        let events = self.events.clone();

        tokio::spawn(async move {
            let report = match removal.execute(&spec).await {
                Ok(removed) => AppEvent::ModelRemoved(removed),
                Err(failure) => AppEvent::ModelRemovalFailed(failure.to_string()),
            };
            let _ = events.send(report);
        });
    }

    /// Starts sweeping the library of everything standing for no model.
    pub fn prune(&self) {
        let pruning = self.pruning.clone();
        let events = self.events.clone();

        tokio::spawn(async move {
            let report = match pruning.execute().await {
                Ok(strays) => AppEvent::LibraryPruned(strays),
                Err(failure) => AppEvent::LibraryPruningFailed(failure.to_string()),
            };
            let _ = events.send(report);
        });
    }
}
