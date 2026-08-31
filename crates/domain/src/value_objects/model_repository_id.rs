use std::fmt;

use crate::errors::DomainError;

/// The namespaced upstream collection that publishes model files, for example
/// `unsloth/Qwen3-8B-GGUF`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelRepositoryId(String);

impl ModelRepositoryId {
    /// Parses an `owner/name` identifier made of two non-empty segments.
    pub fn parse(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        let segments: Vec<&str> = value.split('/').collect();
        if segments.len() == 2 && !segments[0].is_empty() && !segments[1].is_empty() {
            Ok(Self(value))
        } else {
            Err(DomainError::MalformedRepository(value))
        }
    }

    /// The owner (first) segment of the identifier.
    pub fn owner(&self) -> &str {
        self.0.split('/').next().unwrap_or_default()
    }

    /// The model (second) segment of the identifier.
    pub fn name(&self) -> &str {
        self.0.split('/').nth(1).unwrap_or_default()
    }

    /// The identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModelRepositoryId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod model_repository_id_tests {
    use crate::{errors::DomainError, value_objects::ModelRepositoryId};

    #[test]
    fn a_two_segment_repository_identifier_parses() {
        let id = ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id");
        assert_eq!(id.owner(), "unsloth");
        assert_eq!(id.name(), "Qwen3-8B-GGUF");
    }

    #[test]
    fn repository_identifiers_need_exactly_two_non_empty_segments() {
        assert_eq!(
            ModelRepositoryId::parse("no-slash"),
            Err(DomainError::MalformedRepository("no-slash".to_owned()))
        );
        assert_eq!(
            ModelRepositoryId::parse("a/b/c"),
            Err(DomainError::MalformedRepository("a/b/c".to_owned()))
        );
        assert_eq!(
            ModelRepositoryId::parse("/onlyname"),
            Err(DomainError::MalformedRepository("/onlyname".to_owned()))
        );
    }
}
