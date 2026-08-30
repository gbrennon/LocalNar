use domain::{ModelSpec, RemovedModel};

use crate::errors::remove_model_error::RemoveModelError;

/// Inbound contract for discarding one model from the local library.
///
/// This is the authority an install has no counterpart for: the operator
/// reclaims the machine by naming a model and having everything the library
/// keeps for it removed, the bytes and the digest recorded alongside them, so a
/// later read reports the model as absent rather than as an unproven replica.
///
/// The answer names the space that came back, which is what the operator
/// removed the model for. Removal is unconditional on the replica's state: a
/// proven model is as removable as a broken one, since only the operator knows
/// whether they still want it.
///
/// A model the library does not hold is reported as
/// `RemoveModelError::NotInstalled` instead of answering with nothing reclaimed,
/// so an operator is never told space came back that never went. A replica the
/// library still reports after the removal is
/// `RemoveModelError::StillInstalled`.
pub trait RemoveModelPort: Send + Sync {
    /// Discards everything the library holds for `spec`.
    async fn execute(&self, spec: &ModelSpec) -> Result<RemovedModel, RemoveModelError>;
}
