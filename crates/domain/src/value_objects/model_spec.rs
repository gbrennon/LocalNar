use std::{fmt, hash::Hash};

use crate::value_objects::{ModelFileName, ModelRepository, ModelTag};

/// The self-contained operator intent to install one local model.
///
/// A repository paired with a file names exactly one downloadable model, so
/// the pair is the identity; a search result can be turned into this intent
/// without asking the operator for anything further. The tags the model is
/// marked with travel alongside that identity as descriptive capabilities and
/// take no part in it: equality and hashing consider only the repository and
/// file, so two intents for the same model are equal however they were tagged.
#[derive(Clone, Debug)]
pub struct ModelSpec {
    repository: ModelRepository,
    file: ModelFileName,
    tags: Vec<ModelTag>,
}

impl PartialEq for ModelSpec {
    fn eq(&self, other: &Self) -> bool {
        self.repository == other.repository && self.file == other.file
    }
}

impl Eq for ModelSpec {}

impl Hash for ModelSpec {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.repository.hash(state);
        self.file.hash(state);
    }
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
        named_spec_marked_with("org/name", "model.gguf", tags)
    }

    fn named_spec_marked_with(id: &str, file: &str, tags: Vec<ModelTag>) -> ModelSpec {
        let identifier = ModelRepositoryId::parse(id).expect("valid id");
        ModelSpec::new(
            ModelRepository::at_default_revision(identifier),
            ModelFileName::new(file).expect("valid file name"),
            tags,
        )
    }

    fn hash_of(spec: &ModelSpec) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        spec.hash(&mut hasher);
        hasher.finish()
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
    fn the_repository_and_file_name_the_model_are_exposed() {
        let spec = spec_marked_with(Vec::new());

        assert_eq!(spec.repository().identifier().as_str(), "org/name");
        assert_eq!(spec.file().as_str(), "model.gguf");
    }

    #[test]
    fn tags_are_absent_from_the_rendered_identity() {
        let spec = spec_marked_with(vec![ModelTag::new("text-generation").expect("valid tag")]);

        assert_eq!(spec.to_string(), "org/name@main::model.gguf");
    }

    #[test]
    fn two_intents_for_the_same_model_are_equal_however_they_were_tagged() {
        let untagged = spec_marked_with(Vec::new());
        let tagged = spec_marked_with(vec![ModelTag::new("text-generation").expect("valid tag")]);
        let differently_tagged =
            spec_marked_with(vec![ModelTag::new("conversational").expect("valid tag")]);

        assert_eq!(untagged, tagged);
        assert_eq!(tagged, differently_tagged);
        assert_eq!(hash_of(&untagged), hash_of(&tagged));
        assert_eq!(hash_of(&tagged), hash_of(&differently_tagged));
    }

    #[test]
    fn intents_for_different_models_are_not_equal() {
        let spec = named_spec_marked_with("org/name", "model.gguf", Vec::new());
        let other_file = named_spec_marked_with("org/name", "other.gguf", Vec::new());
        let other_repository = named_spec_marked_with("org/other", "model.gguf", Vec::new());

        assert_ne!(spec, other_file);
        assert_ne!(spec, other_repository);
    }
}
