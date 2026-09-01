use localnar_domain::{InstalledModel, ModelSpec};

use crate::errors::install_model_error::InstallModelError;

/// Inbound contract for bringing one model into its verified state.
///
/// The use case honors the domain state machine:
/// - `Verified` → no-op
/// - `Downloaded` → verify, then verify or repair
/// - `Missing` → fetch, then commit, then verify or repair
/// - `IntegrityMismatch` → repair (re-fetch), then verify or repair once
///
/// Success means the bytes are on disk, and the answer says where. Integrity
/// is reported rather than assumed: a replica whose upstream advertised no
/// checksum comes back installed but not verified, which the caller reads
/// through `InstalledModel::is_verified`.
///
/// A single repair attempt is made for an integrity mismatch; if the repaired
/// artifact still mismatches the advertised checksum the call returns
/// `InstallModelError::UnresolvedIntegrity`. If a fetch leaves the replica
/// still absent the call returns `InstallModelError::UpstreamUnavailable`.
pub trait InstallModelPort: Send + Sync {
    /// Drives `spec` towards the verified state.
    async fn execute(&self, spec: &ModelSpec) -> Result<InstalledModel, InstallModelError>;
}
