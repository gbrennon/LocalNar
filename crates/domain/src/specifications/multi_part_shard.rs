use crate::specifications::Specification;
use crate::value_objects::ModelFileName;

/// The rule that a file name marks it as one part of a weight split across files.
pub struct MultiPartShard;

impl MultiPartShard {
    const SHARD_MARKER: &'static str = "of";
    const TOKEN_SEPARATOR: char = '-';
    const MARKED_TOKEN_RUN: usize = 3;

    fn is_ordinal(token: &str) -> bool {
        !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_digit())
    }
}

impl Specification<ModelFileName> for MultiPartShard {
    /// Satisfied when the name carries an `<ordinal>-of-<ordinal>` run.
    ///
    /// The run is looked for in the name without its extension, so the trailing
    /// ordinal is recognized even when the extension follows it directly, as in
    /// `Qwen3-235B-00001-of-00003.gguf`.
    fn is_satisfied_by(&self, candidate: &ModelFileName) -> bool {
        let name = candidate.as_str();
        let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
        let tokens: Vec<&str> = stem.split(Self::TOKEN_SEPARATOR).collect();

        tokens.windows(Self::MARKED_TOKEN_RUN).any(|run| {
            Self::is_ordinal(run[0]) && run[1] == Self::SHARD_MARKER && Self::is_ordinal(run[2])
        })
    }
}

#[cfg(test)]
mod multi_part_shard_tests {
    use crate::specifications::{MultiPartShard, Specification};
    use crate::value_objects::ModelFileName;

    fn is_one_part(name: &str) -> bool {
        let file = ModelFileName::new(name).expect("valid file name");
        MultiPartShard.is_satisfied_by(&file)
    }

    #[test]
    fn a_part_of_a_split_weight_satisfies_the_rule() {
        assert!(is_one_part("Qwen3-235B-Q4_K_M-00001-of-00003.gguf"));
        assert!(is_one_part("Qwen3-235B-Q4_K_M-00003-of-00003.gguf"));
    }

    #[test]
    fn the_run_is_recognized_when_the_extension_follows_the_last_ordinal() {
        assert!(is_one_part("Qwen3-235B-00001-of-00003.gguf"));
    }

    #[test]
    fn a_whole_weight_does_not_satisfy_the_rule() {
        assert!(!is_one_part("Qwen3-8B-Q4_K_M.gguf"));
        assert!(!is_one_part("model.gguf"));
    }

    #[test]
    fn a_run_of_non_numeric_parts_does_not_satisfy_the_rule() {
        assert!(!is_one_part("model-a-of-b.gguf"));
        assert!(!is_one_part("best-of-breed.gguf"));
    }

    #[test]
    fn the_marker_alone_does_not_satisfy_the_rule() {
        assert!(!is_one_part("of.gguf"));
        assert!(!is_one_part("00001-of.gguf"));
    }
}
