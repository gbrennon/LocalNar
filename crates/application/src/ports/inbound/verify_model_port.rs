use domain::{ManagedModel, ModelSpec};

use crate::errors::verify_model_error::VerifyModelError;

/// Inbound contract for re-proving one locally installed model against the
/// digest the library recorded for it.
///
/// A listing reports the state the library recorded when the model was
/// installed; this use case re-reads the bytes and reports what they are now.
/// That is what catches a replica that has since been truncated, corrupted, or
/// edited underneath the library, none of which a recorded digest can notice on
/// its own.
///
/// Nothing upstream is consulted, so verification works offline and answers
/// about the machine alone. A replica the library never proved has no recorded
/// digest to be re-proven against and comes back unproven rather than failing;
/// obtaining a digest for it means installing the model again, which is the
/// install use case's repair path.
///
/// A replica whose bytes disagree with the recorded digest is the verdict, not a
/// failure: it comes back as a `ManagedModel` reporting `is_broken`, carrying
/// both digests, so the operator can choose to repair or discard it.
pub trait VerifyModelPort: Send + Sync {
    /// Re-reads the replica of `spec` and reports the state its bytes prove.
    async fn execute(&self, spec: &ModelSpec) -> Result<ManagedModel, VerifyModelError>;
}
