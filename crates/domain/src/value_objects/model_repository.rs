use std::fmt;

use crate::value_objects::{ModelRepositoryId, ModelRevision};

/// A named upstream repository pinned to one concrete revision.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelRepository {
    identifier: ModelRepositoryId,
    revision: ModelRevision,
}

impl ModelRepository {
    /// Builds a repository pinned to the given revision.
    pub fn new(identifier: ModelRepositoryId, revision: ModelRevision) -> Self {
        Self {
            identifier,
            revision,
        }
    }

    /// Builds a repository at the conventional `main` revision.
    pub fn at_default_revision(identifier: ModelRepositoryId) -> Self {
        Self {
            identifier,
            revision: ModelRevision::default(),
        }
    }

    /// The namespaced upstream identifier.
    pub fn identifier(&self) -> &ModelRepositoryId {
        &self.identifier
    }

    /// The revision this repository is pinned to.
    pub fn revision(&self) -> &ModelRevision {
        &self.revision
    }
}

impl fmt::Display for ModelRepository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}@{}", self.identifier, self.revision)
    }
}

#[cfg(test)]
mod model_repository_tests {
    use crate::value_objects::{ModelRepository, ModelRepositoryId, ModelRevision};

    #[test]
    fn repository_renders_as_identifier_at_revision() {
        let id = ModelRepositoryId::parse("org/name").expect("valid id");
        let revision = ModelRevision::new("refs/pr/7").expect("valid revision");
        let repository = ModelRepository::new(id, revision);
        assert_eq!(repository.to_string(), "org/name@refs/pr/7");
        assert_eq!(repository.revision().as_str(), "refs/pr/7");
    }
}
