use localnar_domain::{Checksum, InstalledModel, ModelArtifact, ModelSpec, ModelState};

use crate::errors::library_error::LibraryError;

/// Outbound contract for the durable, on-disk store of installed model files.
///
/// The application layer owns this abstraction; adapters decide the filesystem
/// layout and hashing while the meanings of `ModelState` stay in the domain.
pub trait ModelLibraryPort: Send + Sync {
    /// Reads the current state of one installed model without side effects.
    async fn installed_state(&self, model: &ModelSpec) -> Result<ModelState, LibraryError>;

    /// Commits a downloaded artifact into the durable place for `model`.
    async fn commit_artifact(
        &self,
        model: &ModelSpec,
        artifact: &ModelArtifact,
    ) -> Result<ModelState, LibraryError>;

    /// Verifies the installed replica for `model` against `expected` checksum.
    ///
    /// With `None` the library may not preemptively fail: whether a missing
    /// digest on the remote side postpones verification is the caller's choice.
    async fn verify_integrity(
        &self,
        model: &ModelSpec,
        expected: Option<Checksum>,
    ) -> Result<ModelState, LibraryError>;

    /// Describes where the replica for `model` currently lives.
    ///
    /// Called once the state machine has settled on a present replica. A
    /// library that claims a replica exists and then cannot describe it is
    /// inconsistent, which it reports as `LibraryError::Unreadable`.
    async fn locate(&self, model: &ModelSpec) -> Result<InstalledModel, LibraryError>;
}
