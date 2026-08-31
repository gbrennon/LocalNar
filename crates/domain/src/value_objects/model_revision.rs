use std::fmt;

use crate::errors::DomainError;

/// A concrete branch, tag, or commit of a repository to resolve against.
///
/// The default is `main`, which is what most GGUF repositories publish under.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelRevision(String);

impl ModelRevision {
    /// The revision name that most repositories expose as their default.
    pub const DEFAULT_REVISION: &'static str = "main";

    /// Builds a revision and rejects blank values.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainError::EmptyRevision);
        }
        Ok(Self(value))
    }

    /// The conventional `main` revision.
    pub fn main() -> Self {
        Self(Self::DEFAULT_REVISION.to_owned())
    }

    /// The revision as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ModelRevision {
    fn default() -> Self {
        Self::main()
    }
}

impl fmt::Display for ModelRevision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod model_revision_tests {
    use crate::{errors::DomainError, value_objects::ModelRevision};

    #[test]
    fn default_revision_is_main() {
        let revision = ModelRevision::default();
        assert_eq!(revision.as_str(), "main");
        assert_eq!(ModelRevision::new(""), Err(DomainError::EmptyRevision));
    }
}
