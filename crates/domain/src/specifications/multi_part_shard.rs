use crate::{specifications::Specification, value_objects::ModelFileName};

/// The rule that a file name marks it as one part of a weight split across files.
pub struct MultiPartShard;

impl MultiPartShard {
    const SHARD_MARKER: &'static str = "of";
    const TOKEN_SEPARATOR: char = '-';
    const MARKED_TOKEN_RUN: usize = 3;

    fn is_ordinal(token: &str) -> bool {
        !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_digit())
    }

    /// True when the name is the first part (`00001-of-...`) of a split weight.
    ///
    /// The first part is the file the loader is pointed at; the remaining parts
    /// are discovered as its siblings, so a repository's split weight is
    /// represented by this one part alone.
    pub fn is_first_part(&self, candidate: &ModelFileName) -> bool {
        Self::parse(candidate.as_str()).is_some_and(|shard| shard.index == 1)
    }

    /// Every part file name of the split weight this name belongs to, in order,
    /// or nothing when the name is not a shard part.
    ///
    /// The `<ordinal>-of-<total>` run fully determines the sibling names, so the
    /// whole set can be reconstructed from any one part without consulting the
    /// catalog again.
    pub fn all_parts(&self, candidate: &ModelFileName) -> Option<Vec<ModelFileName>> {
        let shard = Self::parse(candidate.as_str())?;
        (1..=shard.total)
            .map(|index| {
                let name = format!(
                    "{}{:0width$}{}",
                    shard.prefix,
                    index,
                    shard.suffix,
                    width = shard.width
                );
                ModelFileName::new(name).ok()
            })
            .collect()
    }

    const MARKER: &'static str = "-of-";

    /// Locates the `<ordinal>-of-<ordinal>` run and the text surrounding it, so
    /// the parts can be counted and the sibling names rebuilt.
    fn parse(name: &str) -> Option<Shard> {
        name.match_indices(Self::MARKER)
            .find_map(|(marker_at, _)| Self::shard_at(name, marker_at))
    }

    /// Reads the shard run anchored at the `-of-` found at `marker_at`, or
    /// nothing when the surrounding text is not a genuine ordinal run.
    fn shard_at(name: &str, marker_at: usize) -> Option<Shard> {
        let before = &name[..marker_at];
        let index_start = Self::ordinal_start(before)?;
        let index_text = &before[index_start..];
        let total_text = Self::leading_digits(&name[marker_at + Self::MARKER.len()..]);

        Some(Shard {
            prefix: name[..index_start].to_owned(),
            width: index_text.len(),
            index: index_text.parse().ok()?,
            total: total_text.parse().ok()?,
            suffix: name[marker_at..].to_owned(),
        })
    }

    /// The byte index at which `before` ends in a `-`-separated run of digits,
    /// or nothing when it has no such trailing ordinal.
    fn ordinal_start(before: &str) -> Option<usize> {
        let start = before
            .rfind(|character: char| !character.is_ascii_digit())
            .map_or(0, |boundary| boundary + character_width(before, boundary));
        let has_digits = start < before.len();
        let separated = start > 0 && before.as_bytes()[start - 1] == b'-';
        (has_digits && separated).then_some(start)
    }

    /// The leading run of digits in `after`, empty when it does not begin with
    /// one.
    fn leading_digits(after: &str) -> &str {
        let end = after
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(after.len());
        &after[..end]
    }
}

/// The decomposition of a shard part name: everything before the running
/// ordinal, the ordinal's zero-padded width, its value, the declared total, and
/// the `-of-<total>` tail that closes the run.
struct Shard {
    prefix: String,
    width: usize,
    index: u32,
    total: u32,
    suffix: String,
}

/// The byte width of the character starting at `boundary`, so an index can be
/// advanced past a non-digit of any length.
fn character_width(text: &str, boundary: usize) -> usize {
    text[boundary..].chars().next().map_or(1, char::len_utf8)
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
    use crate::{
        specifications::{MultiPartShard, Specification},
        value_objects::ModelFileName,
    };

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

    fn named(name: &str) -> ModelFileName {
        ModelFileName::new(name).expect("valid file name")
    }

    #[test]
    fn only_the_leading_part_is_the_first_part() {
        assert!(MultiPartShard.is_first_part(&named("Qwen3-235B-Q4_K_M-00001-of-00003.gguf")));
        assert!(!MultiPartShard.is_first_part(&named("Qwen3-235B-Q4_K_M-00002-of-00003.gguf")));
        assert!(!MultiPartShard.is_first_part(&named("Qwen3-8B-Q4_K_M.gguf")));
    }

    #[test]
    fn a_shard_part_expands_to_the_whole_ordered_set() {
        let parts = MultiPartShard
            .all_parts(&named("Qwen3-235B-Q4_K_M-00002-of-00003.gguf"))
            .expect("a shard part expands");

        let names: Vec<&str> = parts.iter().map(ModelFileName::as_str).collect();
        assert_eq!(
            names,
            vec![
                "Qwen3-235B-Q4_K_M-00001-of-00003.gguf",
                "Qwen3-235B-Q4_K_M-00002-of-00003.gguf",
                "Qwen3-235B-Q4_K_M-00003-of-00003.gguf",
            ]
        );
    }

    #[test]
    fn the_padding_width_is_preserved_when_expanding() {
        let parts = MultiPartShard
            .all_parts(&named("model-1-of-2.gguf"))
            .expect("a shard part expands");

        let names: Vec<&str> = parts.iter().map(ModelFileName::as_str).collect();
        assert_eq!(names, vec!["model-1-of-2.gguf", "model-2-of-2.gguf"]);
    }

    #[test]
    fn a_whole_weight_has_no_parts_to_expand() {
        assert!(
            MultiPartShard
                .all_parts(&named("Qwen3-8B-Q4_K_M.gguf"))
                .is_none()
        );
    }
}
