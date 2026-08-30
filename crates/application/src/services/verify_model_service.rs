use domain::{ManagedModel, ModelSpec, ModelState};

use crate::errors::verify_model_error::VerifyModelError;
use crate::ports::inbound::verify_model_port::VerifyModelPort;
use crate::ports::outbound::model_library_port::ModelLibraryPort;

/// The use case that re-proves one locally installed model against the digest
/// the library recorded for it.
///
/// It depends on the library alone, so verification answers with the machine
/// offline and can never re-fetch: proving a replica and replacing it are
/// different decisions, and only the operator makes the second one.
pub struct VerifyModelService<Library>
where
    Library: ModelLibraryPort,
{
    library: Library,
}

impl<Library> VerifyModelService<Library>
where
    Library: ModelLibraryPort,
{
    /// Compose the use case from the library port.
    pub fn new(library: Library) -> Self {
        Self { library }
    }
}

impl<Library> VerifyModelPort for VerifyModelService<Library>
where
    Library: ModelLibraryPort,
{
    /// Re-reads the replica of `spec` and reports the state its bytes prove.
    ///
    /// A replica carrying no recorded digest is answered as unproven without
    /// being re-read: there is nothing to prove it against, and hashing bytes
    /// that no digest will be compared to would cost the operator a full read of
    /// the file for no verdict.
    ///
    /// A replica that disappears midway through is reported as not installed
    /// rather than as a library fault, since that is what the operator now has.
    async fn execute(&self, spec: &ModelSpec) -> Result<ManagedModel, VerifyModelError> {
        let state = self.library.installed_state(spec).await?;

        if matches!(state, ModelState::Missing) {
            return Err(VerifyModelError::NotInstalled {
                model: spec.to_string(),
            });
        }

        let replica = self.library.locate(spec).await?;
        let Some(recorded) = replica.digest() else {
            return Ok(ManagedModel::new(replica, ModelState::Downloaded));
        };

        let proven = self.library.verify_integrity(spec, Some(recorded)).await?;

        if matches!(proven, ModelState::Missing) {
            return Err(VerifyModelError::NotInstalled {
                model: spec.to_string(),
            });
        }

        Ok(ManagedModel::new(replica, proven))
    }
}
