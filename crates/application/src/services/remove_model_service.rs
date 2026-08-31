use domain::{ModelSpec, ModelState, RemovedModel};

use crate::errors::remove_model_error::RemoveModelError;
use crate::ports::inbound::remove_model_port::RemoveModelPort;
use crate::ports::outbound::model_eviction_port::ModelEvictionPort;
use crate::ports::outbound::model_library_port::ModelLibraryPort;

/// The use case that discards one model from the local library.
///
/// The library is read before and after the eviction, which is what turns a
/// destructive call into a reportable outcome: the first read decides whether
/// the operator asked for something absent, the second proves the bytes are
/// really gone instead of trusting that they are.
pub struct RemoveModelService<Library, Eviction>
where
    Library: ModelLibraryPort,
    Eviction: ModelEvictionPort,
{
    library: Library,
    eviction: Eviction,
}

impl<Library, Eviction> RemoveModelService<Library, Eviction>
where
    Library: ModelLibraryPort,
    Eviction: ModelEvictionPort,
{
    /// Compose the use case from the library and eviction ports.
    pub fn new(library: Library, eviction: Eviction) -> Self {
        Self { library, eviction }
    }
}

impl<Library, Eviction> RemoveModelPort for RemoveModelService<Library, Eviction>
where
    Library: ModelLibraryPort,
    Eviction: ModelEvictionPort,
{
    /// Discards everything the library holds for `spec` and reports the space
    /// that came back.
    ///
    /// The replica's state is never a reason to refuse: a proven model is as
    /// removable as a broken one. Only its absence is.
    async fn execute(&self, spec: &ModelSpec) -> Result<RemovedModel, RemoveModelError> {
        if matches!(
            self.library.installed_state(spec).await?,
            ModelState::Missing
        ) {
            return Err(RemoveModelError::NotInstalled {
                model: spec.to_string(),
            });
        }

        let removal = self.eviction.evict(spec).await?;

        if !matches!(
            self.library.installed_state(spec).await?,
            ModelState::Missing
        ) {
            return Err(RemoveModelError::StillInstalled {
                model: spec.to_string(),
            });
        }

        Ok(removal)
    }
}
