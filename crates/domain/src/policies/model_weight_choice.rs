use crate::specifications::{Specification, WholeWeightFile};
use crate::value_objects::{Quantization, RemoteModelFile};

/// The rule that reduces everything a repository offers to one candidate file.
pub struct ModelWeightChoice;

impl ModelWeightChoice {
    const PREFERRED_BIT_WIDTH: u8 = 4;
    const UNKNOWN_PRECISION_DISTANCE: u8 = u8::MAX;

    /// The single offered file that stands for the repository, if any qualifies.
    ///
    /// Qualifying is decided by `WholeWeightFile`, so a repository that offers
    /// nothing installable on its own yields no candidate at all rather than a
    /// row that cannot be installed.
    ///
    /// Among the qualifying files the narrowest precision wins, since a
    /// narrower width keeps a model usable while wider ones only cost disk;
    /// an equal distance is settled in favor of the wider precision, then
    /// the smaller file, then the file name, so the choice never depends on
    /// the order the catalog happened to list files in.
    pub fn among(offered: &[RemoteModelFile]) -> Option<&RemoteModelFile> {
        offered
            .iter()
            .filter(|offer| WholeWeightFile.is_satisfied_by(offer.file()))
            .min_by(|left, right| Self::preference(left).cmp(&Self::preference(right)))
    }

    fn preference(offer: &RemoteModelFile) -> (u8, bool, u64, &str) {
        let (distance, narrower_than_preferred) = match Quantization::for_file(offer.file()) {
            Some(quantization) => (
                quantization.bit_width().abs_diff(Self::PREFERRED_BIT_WIDTH),
                quantization.bit_width() < Self::PREFERRED_BIT_WIDTH,
            ),
            None => (Self::UNKNOWN_PRECISION_DISTANCE, true),
        };

        (
            distance,
            narrower_than_preferred,
            offer.size().bytes(),
            offer.file().as_str(),
        )
    }
}

#[cfg(test)]
mod model_weight_choice_tests {
    use crate::policies::ModelWeightChoice;
    use crate::value_objects::{
        ByteLength, ModelFileName, ModelRepository, ModelRepositoryId, RemoteModelFile,
    };

    fn offered(files: &[(&str, u64)]) -> Vec<RemoteModelFile> {
        let identifier = ModelRepositoryId::parse("unsloth/Qwen3-8B-GGUF").expect("valid id");
        let repository = ModelRepository::at_default_revision(identifier);

        files
            .iter()
            .map(|(name, size)| {
                RemoteModelFile::new(
                    repository.clone(),
                    ModelFileName::new(*name).expect("valid file name"),
                    ByteLength::new(*size),
                    None,
                )
            })
            .collect()
    }

    fn chosen(files: &[(&str, u64)]) -> Option<String> {
        let offered = offered(files);
        ModelWeightChoice::among(&offered).map(|file| file.file().to_string())
    }

    #[test]
    fn documentation_and_configuration_never_qualify() {
        assert_eq!(
            chosen(&[
                (".gitattributes", 3_083),
                ("README.md", 12_000),
                ("config.json", 900),
                ("Qwen3-8B-Q4_K_M.gguf", 5_027_784_064),
            ]),
            Some("Qwen3-8B-Q4_K_M.gguf".to_owned())
        );
    }

    #[test]
    fn a_repository_offering_only_split_parts_yields_no_candidate() {
        assert_eq!(
            chosen(&[
                ("Qwen3-235B-Q4_K_M-00001-of-00003.gguf", 15_000_000_000),
                ("Qwen3-235B-Q4_K_M-00002-of-00003.gguf", 15_000_000_000),
            ]),
            None
        );
    }

    #[test]
    fn the_precision_nearest_four_bits_wins() {
        assert_eq!(
            chosen(&[
                ("Qwen3-8B-BF16.gguf", 16_388_044_384),
                ("Qwen3-8B-Q8_0.gguf", 8_710_000_000),
                ("Qwen3-8B-Q4_K_M.gguf", 5_027_784_064),
                ("Qwen3-8B-Q2_K.gguf", 3_281_733_440),
            ]),
            Some("Qwen3-8B-Q4_K_M.gguf".to_owned())
        );
    }

    #[test]
    fn an_equal_distance_from_four_bits_favors_the_wider_precision() {
        assert_eq!(
            chosen(&[
                ("Qwen3-8B-Q3_K_M.gguf", 4_017_000_000),
                ("Qwen3-8B-Q5_K_M.gguf", 5_850_000_000),
            ]),
            Some("Qwen3-8B-Q5_K_M.gguf".to_owned())
        );
    }

    #[test]
    fn an_equal_precision_is_settled_by_the_smaller_file() {
        assert_eq!(
            chosen(&[
                ("Qwen3-8B-Q4_K_M.gguf", 5_027_784_064),
                ("Qwen3-8B-Q4_K_S.gguf", 4_802_000_000),
            ]),
            Some("Qwen3-8B-Q4_K_S.gguf".to_owned())
        );
    }

    #[test]
    fn a_known_precision_is_preferred_over_an_untagged_weight() {
        assert_eq!(
            chosen(&[
                ("model.gguf", 4_000_000_000),
                ("Qwen3-8B-Q8_0.gguf", 8_710_000_000),
            ]),
            Some("Qwen3-8B-Q8_0.gguf".to_owned())
        );
    }

    #[test]
    fn an_untagged_weight_is_still_a_candidate_when_it_is_the_only_one() {
        assert_eq!(
            chosen(&[("model.gguf", 4_000_000_000)]),
            Some("model.gguf".to_owned())
        );
    }

    #[test]
    fn one_part_of_a_multi_part_weight_never_qualifies() {
        assert_eq!(
            chosen(&[
                ("Qwen3-235B-Q4_K_M-00001-of-00003.gguf", 15_000_000_000),
                ("Qwen3-235B-Q4_K_M-00002-of-00003.gguf", 15_000_000_000),
                ("Qwen3-235B-Q8_0.gguf", 250_000_000_000),
            ]),
            Some("Qwen3-235B-Q8_0.gguf".to_owned())
        );
    }

    #[test]
    fn a_repository_offering_only_multi_part_weights_yields_no_candidate() {
        assert_eq!(
            chosen(&[
                ("Qwen3-235B-Q4_K_M-00001-of-00003.gguf", 15_000_000_000),
                ("Qwen3-235B-Q4_K_M-00002-of-00003.gguf", 15_000_000_000),
            ]),
            None
        );
    }

    #[test]
    fn the_extension_is_matched_regardless_of_case() {
        assert_eq!(
            chosen(&[("Qwen3-8B-Q4_K_M.GGUF", 5_027_784_064)]),
            Some("Qwen3-8B-Q4_K_M.GGUF".to_owned())
        );
    }

    #[test]
    fn the_choice_does_not_depend_on_the_listing_order() {
        let ascending = chosen(&[
            ("Qwen3-8B-Q2_K.gguf", 3_281_733_440),
            ("Qwen3-8B-Q4_K_M.gguf", 5_027_784_064),
            ("Qwen3-8B-BF16.gguf", 16_388_044_384),
        ]);
        let descending = chosen(&[
            ("Qwen3-8B-BF16.gguf", 16_388_044_384),
            ("Qwen3-8B-Q4_K_M.gguf", 5_027_784_064),
            ("Qwen3-8B-Q2_K.gguf", 3_281_733_440),
        ]);

        assert_eq!(ascending, descending);
    }
}
