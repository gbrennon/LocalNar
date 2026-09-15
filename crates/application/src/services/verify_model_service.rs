use localnar_domain::{ManagedModel, ModelSpec, ModelState};

use crate::{
    errors::verify_model_error::VerifyModelError,
    ports::{
        inbound::verify_model_port::VerifyModelPort,
        outbound::{
            model_library_port::ModelLibraryPort,
            remote_model_registry_port::RemoteModelRegistryPort,
        },
    },
};

/// The use case that re-proves one locally installed model against the digest
/// the library recorded for it.
///
/// It uses the local library and remote registry to obtain an expected digest
/// when the library has not recorded one. Proving a replica and replacing it
/// are different decisions, and only the operator makes the second one.
pub struct VerifyModelService<Library, Registry>
where
    Library: ModelLibraryPort,
    Registry: RemoteModelRegistryPort,
{
    library: Library,
    registry: Registry,
}

impl<Library, Registry> VerifyModelService<Library, Registry>
where
    Library: ModelLibraryPort,
    Registry: RemoteModelRegistryPort,
{
    /// Compose the use case from the library and remote registry ports.
    pub fn new(library: Library, registry: Registry) -> Self {
        Self { library, registry }
    }

    /// Rejects verification of a replica the library does not hold.
    async fn ensure_present(&self, spec: &ModelSpec) -> Result<(), VerifyModelError> {
        if matches!(
            self.library.installed_state(spec).await?,
            ModelState::Missing
        ) {
            return Err(VerifyModelError::NotInstalled {
                model: spec.to_string(),
            });
        }
        Ok(())
    }

    /// Rejects a replica that disappeared before it could be proven.
    async fn require_installed(
        &self,
        spec: &ModelSpec,
        state: ModelState,
    ) -> Result<ModelState, VerifyModelError> {
        if matches!(state, ModelState::Missing) {
            return Err(VerifyModelError::NotInstalled {
                model: spec.to_string(),
            });
        }
        Ok(state)
    }
}

impl<Library, Registry> VerifyModelPort for VerifyModelService<Library, Registry>
where
    Library: ModelLibraryPort,
    Registry: RemoteModelRegistryPort,
{
    /// Re-reads the replica of `spec` and reports the state its bytes prove.
    ///
    /// A replica carrying no recorded digest is checked against the remote
    /// registry's advertised checksum. If the remote file has no checksum,
    /// verification returns the replica as unproven.
    ///
    /// A replica that disappears midway through is reported as not installed
    /// rather than as a library fault, since that is what the operator now has.
    async fn execute(&self, spec: &ModelSpec) -> Result<ManagedModel, VerifyModelError> {
        self.ensure_present(spec).await?;

        let replica = self.library.locate(spec).await?;
        let expected = match replica.digest() {
            Some(recorded) => Some(recorded),
            None => self
                .registry
                .resolve_model_file(spec.repository(), spec.file())
                .await?
                .checksum(),
        };
        let Some(expected) = expected else {
            return Ok(ManagedModel::new(replica, ModelState::Downloaded));
        };

        let proven = self.library.verify_integrity(spec, Some(expected)).await?;
        let proven = self.require_installed(spec, proven).await?;

        Ok(ManagedModel::new(replica, proven))
    }
}
