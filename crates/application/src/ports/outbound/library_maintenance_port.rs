use domain::DiscardedStray;

use crate::errors::library_error::LibraryError;

/// Outbound contract for clearing what the library keeps that is not a model.
///
/// The store accumulates leftovers around its replicas: a recorded digest whose
/// model file went away by other means, or a directory left empty once its last
/// model did. None of them is an installed model, so none can be evicted as
/// one, yet they occupy the machine.
///
/// An implementation must only ever discard what stands for no model. A replica,
/// proven or not, is a model the operator installed and is never a leftover, so
/// sweeping the library must never make a model unreachable. The answer names
/// each leftover discarded, so a sweep that found nothing reports an empty one
/// rather than a failure.
pub trait LibraryMaintenancePort: Send + Sync {
    /// Discards every leftover the library keeps that stands for no model.
    async fn discard_strays(&self) -> Result<Vec<DiscardedStray>, LibraryError>;
}
