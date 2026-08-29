use std::fmt;

use crate::model_file_name::ModelFileName;
use crate::model_repository::ModelRepository;

/// The self-contained operator intent to install one local model.
///
/// A repository paired with a file names exactly one downloadable model, so
/// the pair is the identity; a search result can be turned into this intent
/// without asking the operator for anything further.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelSpec {
    repository: ModelRepository,
    file: ModelFileName,
}

impl ModelSpec {
    /// Builds the install intent for one upstream file.
    pub fn new(repository: ModelRepository, file: ModelFileName) -> Self {
        Self { repository, file }
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
        write!(formatter, "{}::{}", self.repository, self.file)
    }
}
