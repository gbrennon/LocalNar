use std::fmt;

use crate::value_objects::{ModelFileName, ModelRepository, ModelTag};

/// The self-contained operator intent to install one local model.
///
/// A repository paired with a file names exactly one downloadable model, so
/// the pair is the identity; a search result can be turned into this intent
/// without asking the operator for anything further. The tags the model is
/// marked with travel alongside that identity as descriptive capabilities and
/// take no part in it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelSpec {
    repository: ModelRepository,
    file: ModelFileName,
    tags: Vec<ModelTag>,
}

impl ModelSpec {
    /// Builds the install intent for one upstream file marked with `tags`.
    pub fn new(repository: ModelRepository, file: ModelFileName, tags: Vec<ModelTag>) -> Self {
        Self {
            repository,
            file,
            tags,
        }
    }

    /// The upstream repository the model is drawn from.
    pub fn repository(&self) -> &ModelRepository {
        &self.repository
    }

    /// The exact repository file to fetch.
    pub fn file(&self) -> &ModelFileName {
        &self.file
    }

    /// The capabilities this model is marked with, empty when none are known.
    pub fn tags(&self) -> &[ModelTag] {
        &self.tags
    }
}

impl fmt::Display for ModelSpec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.repository, self.file)
    }
}

#[cfg(test)]
mod model_spec_tests {
    use crate::value_objects::{
        ModelFileName, ModelRepository, ModelRepositoryId, ModelSpec, ModelTag,
    };

    fn spec_marked_with(tags: Vec<ModelTag>) -> ModelSpec {
        let identifier = ModelRepositoryId::parse("org/name").expect("valid id");
        ModelSpec::new(
            ModelRepository::at_default_revision(identifier),
            ModelFileName::new("model.gguf").expect("valid file name"),
            tags,
        )
    }

    #[test]
    fn the_tags_a_model_is_marked_with_are_retained() {
        let tags = vec![
            ModelTag::new("text-generation").expect("valid tag"),
            ModelTag::new("conversational").expect("valid tag"),
        ];

        let spec = spec_marked_with(tags.clone());

        assert_eq!(spec.tags(), tags.as_slice());
    }

    #[test]
    fn a_model_marked_with_nothing_carries_no_tags() {
        let spec = spec_marked_with(Vec::new());

        assert!(spec.tags().is_empty());
    }

    #[test]
    fn tags_are_absent_from_the_rendered_identity() {
        let spec = spec_marked_with(vec![ModelTag::new("text-generation").expect("valid tag")]);

        assert_eq!(spec.to_string(), "org/name@main::model.gguf");
    }
}
