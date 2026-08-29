use std::fmt;

/// The number of tokens a model can attend to at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextLength(u32);

impl ContextLength {
    const TOKENS_PER_KILO: u32 = 1_024;

    /// Builds a context window from an exact token count.
    pub const fn new(tokens: u32) -> Self {
        Self(tokens)
    }

    /// The exact number of tokens the window holds.
    pub const fn tokens(self) -> u32 {
        self.0
    }
}

impl fmt::Display for ContextLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tokens = self.0;
        if tokens < Self::TOKENS_PER_KILO {
            write!(formatter, "{tokens}")
        } else if tokens.is_multiple_of(Self::TOKENS_PER_KILO) {
            write!(formatter, "{}K", tokens / Self::TOKENS_PER_KILO)
        } else {
            write!(
                formatter,
                "{:.1}K",
                tokens as f64 / Self::TOKENS_PER_KILO as f64
            )
        }
    }
}

#[cfg(test)]
mod context_length_tests {
    use crate::context_length::ContextLength;

    #[test]
    fn whole_kilo_windows_render_without_decimals() {
        assert_eq!(ContextLength::new(40_960).to_string(), "40K");
        assert_eq!(ContextLength::new(131_072).to_string(), "128K");
        assert_eq!(ContextLength::new(1_024).to_string(), "1K");
    }

    #[test]
    fn partial_kilo_windows_render_with_one_decimal() {
        assert_eq!(ContextLength::new(1_536).to_string(), "1.5K");
    }

    #[test]
    fn windows_below_one_kilo_render_exactly() {
        assert_eq!(ContextLength::new(512).to_string(), "512");
    }

    #[test]
    fn the_exact_token_count_survives_rendering() {
        assert_eq!(ContextLength::new(40_960).tokens(), 40_960);
    }
}
