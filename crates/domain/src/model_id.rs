use std::fmt;

use crate::domain_error::DomainError;

/// A short, human-oriented identifier for a locally managed model.
///
/// It names the local installation entry (and its install directory), not the
/// upstream repository; for example `qwen3-8b` identifies the downloaded file
/// even though it comes from `unsloth/Qwen3-8B-GGUF`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelId(String);

impl ModelId {
    /// Builds an identifier and rejects values that are completely blank.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainError::EmptyModelId);
        }
        Ok(Self(value))
    }

    /// The identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModelId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod model_id_tests {
    use crate::domain_error::DomainError;
    use crate::model_id::ModelId;

    #[test]
    fn a_non_blank_identifier_is_accepted() {
        let id = ModelId::new("qwen3-8b").expect("valid identifier");
        assert_eq!(id.as_str(), "qwen3-8b");
    }

    #[test]
    fn a_blank_identifier_is_rejected() {
        assert_eq!(ModelId::new("   "), Err(DomainError::EmptyModelId));
        assert_eq!(ModelId::new(""), Err(DomainError::EmptyModelId));
    }
}
