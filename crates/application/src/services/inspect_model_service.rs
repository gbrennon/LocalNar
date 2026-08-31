use domain::{ManagedModel, ModelSpec, ModelState};

use crate::errors::inspect_model_error::InspectModelError;
use crate::ports::inbound::inspect_model_port::InspectModelPort;
use crate::ports::outbound::model_library_port::ModelLibraryPort;

/// The use case that describes one locally installed model in full.
///
/// It reads the library twice and writes nothing: once for the state the library
/// holds the model in, once for the place the bytes occupy. The two readings are
/// what the operator needs together, and pairing them here keeps the library
/// port free of a method that exists only to serve this view.
pub struct InspectModelService<Library>
where
    Library: ModelLibraryPort,
{
    library: Library,
}

impl<Library> InspectModelService<Library>
where
    Library: ModelLibraryPort,
{
    /// Compose the use case from the library port.
    pub fn new(library: Library) -> Self {
        Self { library }
    }
}

impl<Library> InspectModelPort for InspectModelService<Library>
where
    Library: ModelLibraryPort,
{
    /// Describes the replica the library holds for `spec`, refusing to describe
    /// a model the library does not hold.
    async fn execute(&self, spec: &ModelSpec) -> Result<ManagedModel, InspectModelError> {
        let state = self.library.installed_state(spec).await?;

        if matches!(state, ModelState::Missing) {
            return Err(InspectModelError::NotInstalled {
                model: spec.to_string(),
            });
        }

        Ok(ManagedModel::new(self.library.locate(spec).await?, state))
    }
}
