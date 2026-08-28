use std::fmt;

use crate::model_file_name::ModelFileName;
use crate::model_id::ModelId;
use crate::model_repository::ModelRepository;

/// The full, self-contained operator intent to install one local model.
///
/// It couples the local identity with the exact upstream location and file so
/// that a discriminator and its plan have everything needed for automation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelSpec {
    id: ModelId,
    repository: ModelRepository,
    file: ModelFileName,
}

impl ModelSpec {
    /// Builds the installment intent for one model.
    pub fn new(id: ModelId, repository: ModelRepository, file: ModelFileName) -> Self {
        Self {
            id,
            repository,
            file,
        }
    }

    /// The local identifier of the model.
    pub fn id(&self) -> &ModelId {
        &self.id
    }

    /// The upstream repository the model is drawn from.
    pub fn repository(&self) -> &ModelRepository {
        &self.repository
    }

    /// The exact repository file to fetch.
    pub fn file(&self) -> &ModelFileName {
        &self.file
    }
}

impl fmt::Display for ModelSpec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} from {}::{}",
            self.id, self.repository, self.file
        )
    }
}
