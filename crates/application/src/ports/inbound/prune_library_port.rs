use domain::DiscardedStray;

use crate::errors::prune_library_error::PruneLibraryError;

/// Inbound contract for clearing what the library keeps that is not a model.
///
/// A library gathers leftovers around its replicas: a recorded digest whose
/// model file went away by other means, a directory left empty once its last
/// model did. They never appear in a listing, because none of them is a model,
/// which is exactly why an operator cannot reclaim them one at a time.
///
/// The sweep is safe by construction: only entries standing for no model are
/// discarded, so no installed model, proven or not, is ever made unreachable.
/// The answer names each leftover and the space it gave back, and a library with
/// nothing to sweep answers with none rather than failing.
pub trait PruneLibraryPort: Send + Sync {
    /// Discards every leftover the library keeps that stands for no model.
    async fn execute(&self) -> Result<Vec<DiscardedStray>, PruneLibraryError>;
}
