use crate::{
    specifications::{MultiPartShard, Specification},
    value_objects::ModelFileName,
};

/// A weight file that is not split across multiple parts.
///
/// This is the domain's structural check: a whole weight is one whose
/// name does not carry the `<ordinal>-of-<ordinal>` marker. Whether the
/// file's extension is actually installable is enforced by the
/// infrastructure that supplies the candidate list.
pub struct WholeWeightFile;

impl Specification<ModelFileName> for WholeWeightFile {
    fn is_satisfied_by(&self, candidate: &ModelFileName) -> bool {
        !MultiPartShard.is_satisfied_by(candidate) && !Self::is_projector(candidate)
    }
}

impl WholeWeightFile {
    fn is_projector(candidate: &ModelFileName) -> bool {
        let name = candidate.as_str();
        let basename = name.rsplit('/').next().unwrap_or(name);
        basename.to_ascii_lowercase().starts_with("mmproj")
    }
}

#[cfg(test)]
mod whole_weight_file_tests {
    use crate::{
        specifications::{Specification, WholeWeightFile},
        value_objects::ModelFileName,
    };

    fn is_whole(name: &str) -> bool {
        let file = ModelFileName::new(name).expect("valid file name");
        WholeWeightFile.is_satisfied_by(&file)
    }

    #[test]
    fn a_whole_weight_file_satisfies_the_rule() {
        assert!(is_whole("Qwen3-8B-Q4_K_M.gguf"));
        assert!(is_whole("model.gguf"));
        assert!(is_whole("README.md"));
    }

    #[test]
    fn one_part_of_a_split_weight_is_excluded() {
        assert!(!is_whole("Qwen3-235B-Q4_K_M-00001-of-00003.gguf"));
        assert!(!is_whole("Qwen3-235B-Q4_K_M-00001-of-00003.safetensors"));
    }

    #[test]
    fn a_split_part_is_excluded_regardless_of_extension() {
        assert!(!is_whole("notes-00001-of-00003.md"));
    }

    #[test]
    fn auxiliary_projector_files_are_excluded() {
        assert!(!is_whole("mmproj-F16.gguf"));
        assert!(!is_whole("mmproj-BF16.gguf"));
    }
}
