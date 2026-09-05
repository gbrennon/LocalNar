use std::fmt;

use crate::value_objects::ModelFileName;

/// The weight precision a published model file was quantized to.
///
/// The label is the tag the publisher used, normalized to upper case, and the
/// bit width is the precision that tag denotes; `Q4_K_M` is therefore four bits
/// and `BF16` is sixteen.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quantization {
    label: String,
    bit_width: u8,
}

impl Quantization {
    const TAG_SEPARATORS: [char; 2] = ['-', '.'];
    const TAG_PREFIXES: [&'static str; 5] = ["BF", "IQ", "TQ", "Q", "F"];
    const TAG_REMAINDER_SEPARATOR: char = '_';

    /// Reads the quantization a file name advertises, when it advertises one.
    ///
    /// Publishers encode the tag as its own dash- or dot-separated token, as in
    /// `Qwen3-8B-Q4_K_M.gguf` or `Mistral-7B-v0.3.IQ4_XS.gguf`, and the tag
    /// closest to the extension is the file's own. A name that carries no such
    /// token, like `model.gguf`, discloses no quantization, and the caller must
    /// treat the precision as unknown rather than assume one.
    pub fn for_file(file: &ModelFileName) -> Option<Self> {
        file.as_str()
            .split(Self::TAG_SEPARATORS)
            .rev()
            .find_map(Self::parse_tag)
    }

    /// The tag the publisher used, in upper case.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// The number of bits the tag spends on each weight.
    pub fn bit_width(&self) -> u8 {
        self.bit_width
    }

    fn parse_tag(token: &str) -> Option<Self> {
        let label = token.to_ascii_uppercase();
        let after_prefix = Self::TAG_PREFIXES
            .iter()
            .find_map(|prefix| label.strip_prefix(prefix))?;

        let digit_count = after_prefix
            .chars()
            .take_while(char::is_ascii_digit)
            .count();
        let bit_width = after_prefix[..digit_count].parse::<u8>().ok()?;

        Self::has_valid_remainder(&after_prefix[digit_count..]).then_some(Self { label, bit_width })
    }

    fn has_valid_remainder(remainder: &str) -> bool {
        remainder.is_empty() || remainder.starts_with(Self::TAG_REMAINDER_SEPARATOR)
    }
}

impl fmt::Display for Quantization {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label)
    }
}

#[cfg(test)]
mod quantization_tests {
    use crate::value_objects::{ModelFileName, Quantization};

    fn quantization_of(name: &str) -> Option<Quantization> {
        Quantization::for_file(&ModelFileName::new(name).expect("valid file name"))
    }

    #[test]
    fn a_dash_separated_tag_is_read_from_the_file_name() {
        let quantization = quantization_of("Qwen3-8B-Q4_K_M.gguf").expect("a tagged file name");

        assert_eq!(quantization.label(), "Q4_K_M");
        assert_eq!(quantization.bit_width(), 4);
    }

    #[test]
    fn a_dot_separated_tag_is_read_from_the_file_name() {
        let quantization =
            quantization_of("Mistral-7B-v0.3.IQ4_XS.gguf").expect("a tagged file name");

        assert_eq!(quantization.label(), "IQ4_XS");
        assert_eq!(quantization.bit_width(), 4);
    }

    #[test]
    fn a_lower_case_tag_is_normalized_to_upper_case() {
        let quantization = quantization_of("model-q8_0.gguf").expect("a tagged file name");

        assert_eq!(quantization.label(), "Q8_0");
        assert_eq!(quantization.bit_width(), 8);
    }

    #[test]
    fn floating_point_tags_carry_their_full_width() {
        assert_eq!(
            quantization_of("Qwen3-8B-BF16.gguf")
                .expect("a tagged file name")
                .bit_width(),
            16
        );
        assert_eq!(
            quantization_of("Qwen3-8B-F32.gguf")
                .expect("a tagged file name")
                .bit_width(),
            32
        );
    }

    #[test]
    fn the_tag_closest_to_the_extension_wins() {
        let quantization = quantization_of("Q8_0-repack-Q4_K_S.gguf").expect("a tagged file name");

        assert_eq!(quantization.label(), "Q4_K_S");
    }

    #[test]
    fn a_file_name_without_a_tag_discloses_no_quantization() {
        assert_eq!(quantization_of("model.gguf"), None);
        assert_eq!(quantization_of("README.md"), None);
    }

    #[test]
    fn parameter_and_name_tokens_are_not_mistaken_for_tags() {
        assert_eq!(quantization_of("Falcon3-8B.gguf"), None);
        assert_eq!(quantization_of("gemma-3-270m.gguf"), None);
    }

    #[test]
    fn a_tag_renders_as_its_label() {
        let quantization = quantization_of("Qwen3-8B-Q5_K_M.gguf").expect("a tagged file name");

        assert_eq!(quantization.to_string(), "Q5_K_M");
    }
}
