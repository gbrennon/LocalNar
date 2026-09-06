use crate::value_objects::{
    ByteLength, Checksum, ModelFileName, ModelRepository, ModelSpec, ModelTag,
};

/// One file offered by a remote repository, described as the domain sees it.
///
/// A repository publishes many files; this is the one an adapter has chosen to
/// stand for an installable model. It carries only what the domain reasons
/// about: where the file lives (`repository` + `file`), what installing it
/// costs (`size`), whether its integrity can be proven (`checksum`), and the
/// capability tags the model is marked with (`tags`).
///
/// Tags describe the model, not the byte stream, so they are attached to the
/// file the adapter elects as the model's representative rather than to every
/// published file. Free-form catalog metadata the domain never reasons about
/// (for example `likes` or download counts) stays in the infrastructure
/// adapters and never reaches this value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteModelFile {
    repository: ModelRepository,
    file: ModelFileName,
    size: ByteLength,
    checksum: Option<Checksum>,
    tags: Vec<ModelTag>,
}

impl RemoteModelFile {
    /// Describes an offered file, marked with no capability tags.
    ///
    /// Tags are opt-in through [`RemoteModelFile::with_tags`], so an adapter
    /// that has none to report yields a file that is still fully actionable.
    pub fn new(
        repository: ModelRepository,
        file: ModelFileName,
        size: ByteLength,
        checksum: Option<Checksum>,
    ) -> Self {
        Self {
            repository,
            file,
            size,
            checksum,
            tags: Vec::new(),
        }
    }

    /// Marks the file with the capability `tags` the catalog disclosed.
    ///
    /// The tags travel with the install intent this file produces, so a model
    /// installed from a search result keeps the capabilities it was found by.
    /// This replaces any tags already set rather than adding to them, so an
    /// adapter states the whole capability set in a single call.
    pub fn with_tags(mut self, tags: Vec<ModelTag>) -> Self {
        self.tags = tags;
        self
    }

    /// The upstream repository the file is drawn from.
    pub fn repository(&self) -> &ModelRepository {
        &self.repository
    }

    /// The name of the file within the repository.
    pub fn file(&self) -> &ModelFileName {
        &self.file
    }

    /// The number of bytes the file occupies.
    pub fn size(&self) -> ByteLength {
        self.size
    }

    /// The capabilities the model is marked with, empty when none are known.
    pub fn tags(&self) -> &[ModelTag] {
        &self.tags
    }

    /// The install intent that downloading this file would satisfy.
    ///
    /// A search result is therefore directly actionable: the operator picks a
    /// row and the intent follows from it, with nothing further to supply. The
    /// capability tags travel into the intent, so an installed model carries the
    /// same capabilities it was discovered by.
    pub fn to_spec(&self) -> ModelSpec {
        ModelSpec::new(
            self.repository.clone(),
            self.file.clone(),
            self.tags.clone(),
        )
    }

    /// The digest the registry advertised, when it disclosed one.
    pub fn checksum(&self) -> Option<Checksum> {
        self.checksum
    }
}

#[cfg(test)]
mod remote_model_file_tests {
    use crate::value_objects::{
        ByteLength, Checksum, ModelFileName, ModelRepository, ModelRepositoryId, ModelTag,
        RemoteModelFile,
    };

    fn file() -> RemoteModelFile {
        let identifier = ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id");
        RemoteModelFile::new(
            ModelRepository::at_default_revision(identifier),
            ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file name"),
            ByteLength::new(5_027_784_064),
            None,
        )
    }

    #[test]
    fn a_file_exposes_the_facts_the_domain_reasons_about() {
        let digest = Checksum::from_bytes([0x22; 32]);
        let identifier = ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id");
        let offered = RemoteModelFile::new(
            ModelRepository::at_default_revision(identifier),
            ModelFileName::new("Qwen3-8B-Q4_K_M.gguf").expect("valid file name"),
            ByteLength::new(5_027_784_064),
            Some(digest),
        );

        assert_eq!(
            offered.repository().identifier().as_str(),
            "unsloth/Qwen3-8B-GGUF"
        );
        assert_eq!(offered.file().as_str(), "Qwen3-8B-Q4_K_M.gguf");
        assert_eq!(offered.size(), ByteLength::new(5_027_784_064));
        assert_eq!(offered.checksum(), Some(digest));
    }

    #[test]
    fn a_file_is_marked_with_no_tags_by_default() {
        assert!(file().tags().is_empty());
        assert_eq!(file().checksum(), None);
    }

    #[test]
    fn the_tags_a_file_is_marked_with_are_retained() {
        let tags = vec![
            ModelTag::new("text-generation").expect("valid tag"),
            ModelTag::new("conversational").expect("valid tag"),
        ];

        let marked = file().with_tags(tags.clone());

        assert_eq!(marked.tags(), tags.as_slice());
    }

    #[test]
    fn the_install_intent_carries_the_files_tags() {
        let tags = vec![ModelTag::new("text-generation").expect("valid tag")];

        let spec = file().with_tags(tags.clone()).to_spec();

        assert_eq!(spec.tags(), tags.as_slice());
    }

    #[test]
    fn the_install_intent_of_an_unmarked_file_carries_no_tags() {
        assert!(file().to_spec().tags().is_empty());
    }
}
