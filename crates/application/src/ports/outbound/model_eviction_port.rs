use localnar_domain::{ModelSpec, RemovedModel};

use crate::errors::library_error::LibraryError;

/// Outbound contract for discarding one replica from the durable library.
///
/// Eviction is deliberately separate from storage: committing bytes and
/// destroying them are opposite authorities, and only the manager use cases are
/// given the second one.
///
/// An implementation must leave nothing of the model behind, including whatever
/// it recorded alongside the bytes to describe them, so a later read reports the
/// model as `Missing` rather than as an unproven replica. The answer names the
/// space that came back, which is the size the replica occupied before it went.
///
/// Evicting a model the library does not hold is not this port's decision to
/// refuse: it reports the removal of nothing, having reclaimed nothing, and the
/// use case decides whether the operator asked for something absent.
pub trait ModelEvictionPort: Send + Sync {
    /// Discards the replica of `model`, along with everything recorded for it.
    async fn evict(&self, model: &ModelSpec) -> Result<RemovedModel, LibraryError>;
}
