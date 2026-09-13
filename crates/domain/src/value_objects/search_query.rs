use crate::errors::DomainError;

/// A free-text phrase an operator types to discover downloadable models.
///
/// The value is trimmed and may be empty, allowing the catalog to provide its
/// default model list when no search phrase is entered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchQuery(String);

impl SearchQuery {
    /// Builds a query from trimmed text, including the empty catalog query.
    pub fn new(phrase: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self(phrase.into().trim().to_string()))
    }

    /// The trimmed phrase to send upstream.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod search_query_tests {
    use crate::value_objects::SearchQuery;

    #[test]
    fn a_phrase_is_trimmed() {
        let query = SearchQuery::new("  qwen3 gguf  ").expect("the phrase carries text");

        assert_eq!(query.as_str(), "qwen3 gguf");
    }

    #[test]
    fn a_blank_phrase_is_preserved_as_the_catalog_query() {
        let query = SearchQuery::new("   ").expect("the catalog query is valid");

        assert_eq!(query.as_str(), "");
    }
}
