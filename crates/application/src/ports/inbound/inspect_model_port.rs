use domain::{ManagedModel, ModelSpec};

use crate::errors::inspect_model_error::InspectModelError;

/// Inbound contract for reading one locally installed model in full.
///
/// Where a listing describes the whole library at once, this answers about the
/// single model the operator named: where its bytes are, how much space they
/// take, the digest recorded for them, and the state the library holds it in.
///
/// The reading is taken from what the library already recorded, so the call
/// changes nothing and touches no network. A model the library does not hold is
/// reported as `InspectModelError::NotInstalled` rather than as an absent
/// replica, because an operator inspecting a model has already assumed it is
/// there.
pub trait InspectModelPort: Send + Sync {
    /// Describes the replica the library holds for `spec`.
    async fn execute(&self, spec: &ModelSpec) -> Result<ManagedModel, InspectModelError>;
}
