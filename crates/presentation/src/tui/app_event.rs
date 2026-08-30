use domain::{
    DiscardedStray, InstalledModel, ManagedModel, ModelInfo, ModelInventory, RemovedModel,
};

/// Events that can be sent between the TUI components and async tasks.
///
/// Every use case the operator can drive reports its own outcome and its own
/// failure. A failure is never folded into a neighbour's, because the interface
/// returns the operator to a different place depending on what they were doing.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Search completed with one described model per catalog entry
    SearchCompleted(Vec<ModelInfo>),
    /// Search failed with error message
    SearchFailed(String),
    /// Install started
    InstallStarted,
    /// Install progress update (0.0-1.0 ratio, message)
    InstallProgress(f64, String),
    /// Install completed with installed model
    InstallCompleted(InstalledModel),
    /// Install failed with error message
    InstallFailed(String),
    /// The local library was read and holds exactly this listing
    LibraryListed(ModelInventory),
    /// The local library could not be listed
    LibraryListingFailed(String),
    /// One installed model was described in full
    ModelInspected(ManagedModel),
    /// One installed model could not be described
    ModelInspectionFailed(String),
    /// One installed model was re-read and its bytes prove this state
    ModelVerified(ManagedModel),
    /// One installed model could not be verified
    ModelVerificationFailed(String),
    /// One installed model was discarded from the local library
    ModelRemoved(RemovedModel),
    /// One installed model could not be discarded
    ModelRemovalFailed(String),
    /// The library was swept of everything standing for no model
    LibraryPruned(Vec<DiscardedStray>),
    /// The library could not be swept
    LibraryPruningFailed(String),
    /// Quit application
    Quit,
}
