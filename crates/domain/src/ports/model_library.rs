use async_trait::async_trait;

use crate::model_artifact::ModelArtifact;
use crate::model_spec::ModelSpec;
use crate::model_state::ModelState;
use crate::ports::library_error::LibraryError;
use crate::sha256::Sha256;

/// Contract for the durable, on-disk store of installed model files.
///
/// The domain layer dictates that install decisions flow exclusively through
/// these operations; adapters own the filesystem layout and hashing while the
/// domain owns the meanings of `ModelState` and integrity checking.
#[async_trait]
pub trait ModelLibrary: Send + Sync {
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
        expected: Option<Sha256>,
    ) -> Result<ModelState, LibraryError>;
}
