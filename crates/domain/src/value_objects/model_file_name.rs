use std::fmt;

use crate::errors::DomainError;

/// The single, plain name of one file inside a model repository.
///
/// Repositories expose files at their root, so a file name must not contain
/// sub-directories, a leading slash, or traversal components; for example
/// `Qwen3-8B-Q4_K_M.gguf`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelFileName(String);

impl ModelFileName {
    /// Builds a file name and rejects empty, absolute, or traversing values.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.trim().is_empty()
            || value.starts_with('/')
            || value.split('/').any(|segment| segment == "..")
        {
            return Err(DomainError::InvalidFileName(value));
        }
        Ok(Self(value))
    }

    /// The file name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModelFileName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod model_file_name_tests {
    use crate::{errors::DomainError, value_objects::ModelFileName};

    #[test]
    fn a_plain_relative_file_name_is_accepted() {
        let file = ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file name");
        assert_eq!(file.as_str(), "Qwen3-8B-Q4_K_M.gguf");
    }

    #[test]
    fn blank_absolute_and_traversing_names_are_rejected() {
        assert_eq!(
            ModelFileName::new(""),
            Err(DomainError::InvalidFileName(String::new()))
        );
        assert_eq!(
            ModelFileName::new("/absolute.gguf"),
            Err(DomainError::InvalidFileName("/absolute.gguf".to_owned()))
        );
        assert_eq!(
            ModelFileName::new("../escape.gguf"),
            Err(DomainError::InvalidFileName("../escape.gguf".to_owned()))
        );
    }
}
