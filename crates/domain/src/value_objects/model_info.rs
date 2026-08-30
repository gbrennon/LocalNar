use crate::value_objects::{
    ByteLength, ModelProfile, ModelRepositoryId, ModelSpec, Quantization, RemoteModelFile,
};

/// One catalog entry described as a single candidate an operator can act on.
///
/// A repository publishes many files, but only one of them stands for the model
/// an operator means to install, so this value describes that one file: what the
/// model is called, what installing it costs, and at what precision. It is built
/// from the chosen weight file alone, so its size and quantization can never
/// disagree with the install intent it carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelInfo {
    spec: ModelSpec,
    size: ByteLength,
    quantization: Option<Quantization>,
    profile: ModelProfile,
}

impl ModelInfo {
    /// Describes the model that a chosen weight file stands for.
    pub fn describing(weight: &RemoteModelFile, profile: ModelProfile) -> Self {
        Self {
            spec: weight.to_spec(),
            size: weight.size(),
            quantization: Quantization::for_file(weight.file()),
            profile,
        }
    }

    /// The namespaced name the catalog publishes the model under.
    pub fn name(&self) -> &ModelRepositoryId {
        self.spec.repository().identifier()
    }

    /// The install intent this description would satisfy.
    pub fn spec(&self) -> &ModelSpec {
        &self.spec
    }

    /// The bytes the chosen weight file occupies.
    pub fn size(&self) -> ByteLength {
        self.size
    }

    /// The precision of the chosen weight file, when its name disclosed one.
    pub fn quantization(&self) -> Option<&Quantization> {
        self.quantization.as_ref()
    }

    /// What the catalog disclosed about serving the model.
    pub fn profile(&self) -> &ModelProfile {
        &self.profile
    }
}

#[cfg(test)]
mod model_info_tests {
    use crate::value_objects::{
        ByteLength, ContextLength, ModelFileName, ModelInfo, ModelProfile, ModelRepository,
        ModelRepositoryId, ParameterCount, RemoteModelFile,
    };

    fn weight(file_name: &str, size: u64) -> RemoteModelFile {
        let identifier = ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id");
        RemoteModelFile::new(
            ModelRepository::at_default_revision(identifier),
            ModelFileName::new(file_name).expect("valid file name"),
            ByteLength::new(size),
            None,
        )
    }

    #[test]
    fn a_description_is_named_after_the_publishing_repository() {
        let info = ModelInfo::describing(
            &weight("Qwen3-8B-Q4_K_M.gguf", 5_027_784_064),
            ModelProfile::UNDISCLOSED,
        );

        assert_eq!(info.name().as_str(), "unsloth/Qwen3-8B-GGUF");
    }

    #[test]
    fn a_description_carries_the_size_and_precision_of_the_chosen_weight() {
        let info = ModelInfo::describing(
            &weight("Qwen3-8B-Q4_K_M.gguf", 5_027_784_064),
            ModelProfile::UNDISCLOSED,
        );

        assert_eq!(info.size(), ByteLength::new(5_027_784_064));
        assert_eq!(
            info.quantization().map(|quantization| quantization.label()),
            Some("Q4_K_M")
        );
    }

    #[test]
    fn a_description_of_an_untagged_weight_discloses_no_precision() {
        let info = ModelInfo::describing(&weight("model.gguf", 4_000), ModelProfile::UNDISCLOSED);

        assert_eq!(info.quantization(), None);
    }

    #[test]
    fn a_description_is_actionable_as_an_install_intent() {
        let weight = weight("Qwen3-8B-Q4_K_M.gguf", 5_027_784_064);
        let info = ModelInfo::describing(&weight, ModelProfile::UNDISCLOSED);

        assert_eq!(info.spec(), &weight.to_spec());
    }

    #[test]
    fn a_description_carries_the_serving_profile_it_was_given() {
        let profile = ModelProfile::new(
            Some(ParameterCount::new(8_190_735_360)),
            Some(ContextLength::new(40_960)),
        );
        let info = ModelInfo::describing(&weight("Qwen3-8B-Q4_K_M.gguf", 5_027_784_064), profile);

        assert_eq!(
            info.profile().parameters(),
            Some(ParameterCount::new(8_190_735_360))
        );
        assert_eq!(
            info.profile().context_length(),
            Some(ContextLength::new(40_960))
        );
    }
}
