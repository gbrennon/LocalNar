use crate::errors::DomainError;

/// A free-text phrase an operator types to discover downloadable models.
///
/// The value is trimmed and never blank, so an adapter can hand it to an
/// upstream catalog without re-validating it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchQuery(String);

impl SearchQuery {
    /// Builds a query, rejecting a phrase that carries no searchable text.
    pub fn new(phrase: impl Into<String>) -> Result<Self, DomainError> {
        let phrase = phrase.into().trim().to_string();

        if phrase.is_empty() {
            return Err(DomainError::BlankSearchQuery);
        }

        Ok(Self(phrase))
    }

    /// The trimmed phrase to send upstream.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod search_query_tests {
    use crate::errors::DomainError;
    use crate::value_objects::SearchQuery;

    #[test]
    fn a_phrase_is_trimmed() {
        let query = SearchQuery::new("  qwen3 gguf  ").expect("the phrase carries text");

        assert_eq!(query.as_str(), "qwen3 gguf");
    }

    #[test]
    fn a_blank_phrase_is_rejected() {
        assert_eq!(
            SearchQuery::new("   ").unwrap_err(),
            DomainError::BlankSearchQuery
        );
    }
}
