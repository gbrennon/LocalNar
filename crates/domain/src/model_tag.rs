use std::fmt;

use crate::domain_error::DomainError;

/// A single capability label a model is marked with.
///
/// The vocabulary is deliberately open: any non-blank label is a valid tag, so
/// the domain stays free of any one catalog's taxonomy. A registry that wants
/// to constrain the vocabulary enforces that in its own adapter.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelTag(String);

impl ModelTag {
    /// Builds a tag and rejects blank labels.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DomainError::InvalidModelTag);
        }
        Ok(Self(value))
    }

    /// The label as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModelTag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod model_tag_tests {
    use crate::domain_error::DomainError;
    use crate::model_tag::ModelTag;

    #[test]
    fn a_capability_label_is_accepted_verbatim() {
        let tag = ModelTag::new("text-generation").expect("valid tag");

        assert_eq!(tag.as_str(), "text-generation");
        assert_eq!(tag.to_string(), "text-generation");
    }

    #[test]
    fn an_arbitrary_vocabulary_is_accepted() {
        let tag = ModelTag::new("some-registry-specific-label").expect("valid tag");

        assert_eq!(tag.as_str(), "some-registry-specific-label");
    }

    #[test]
    fn empty_and_whitespace_only_labels_are_rejected() {
        assert_eq!(ModelTag::new(""), Err(DomainError::InvalidModelTag));
        assert_eq!(ModelTag::new("   "), Err(DomainError::InvalidModelTag));
        assert_eq!(ModelTag::new("\t\n"), Err(DomainError::InvalidModelTag));
    }
}
