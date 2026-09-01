use localnar_domain::ModelInventory;

use crate::errors::list_installed_models_error::ListInstalledModelsError;

/// Inbound contract for seeing every model this machine holds.
///
/// This is the operator's entry point into managing the library: the answer
/// names where the library lives and describes each replica in it, so a listing
/// is enough to decide what to verify, keep, or discard without a second call
/// per model.
///
/// Nothing upstream is consulted and nothing on disk changes, so the use case
/// answers with the machine offline. Each entry carries the state the library
/// already recorded, which distinguishes a proven replica from one that was
/// never proven but never disproves either; asking for the bytes to be re-read
/// is the verification use case.
///
/// A library holding no model answers with an empty inventory, never an error.
pub trait ListInstalledModelsPort: Send + Sync {
    /// Describes the local library and every replica it holds.
    async fn execute(&self) -> Result<ModelInventory, ListInstalledModelsError>;
}
