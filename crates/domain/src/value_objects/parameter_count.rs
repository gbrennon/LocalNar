use std::fmt;

/// The number of weights a model carries.
///
/// The count is exact; rendering is what abbreviates it, so no precision is
/// lost by holding the value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParameterCount(u64);

impl ParameterCount {
    const BILLION: u64 = 1_000_000_000;
    const MILLION: u64 = 1_000_000;

    /// Builds a count from an exact number of weights.
    pub const fn new(count: u64) -> Self {
        Self(count)
    }

    /// The exact number of weights.
    pub const fn count(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ParameterCount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.0;
        if count >= Self::BILLION {
            write!(formatter, "{:.1}B", count as f64 / Self::BILLION as f64)
        } else if count >= Self::MILLION {
            write!(formatter, "{:.0}M", count as f64 / Self::MILLION as f64)
        } else {
            write!(formatter, "{count}")
        }
    }
}

#[cfg(test)]
mod parameter_count_tests {
    use crate::value_objects::ParameterCount;

    #[test]
    fn billions_render_with_one_decimal() {
        assert_eq!(ParameterCount::new(8_190_735_360).to_string(), "8.2B");
        assert_eq!(ParameterCount::new(1_000_000_000).to_string(), "1.0B");
    }

    #[test]
    fn millions_render_without_decimals() {
        assert_eq!(ParameterCount::new(270_000_000).to_string(), "270M");
        assert_eq!(ParameterCount::new(1_500_000).to_string(), "2M");
    }

    #[test]
    fn smaller_counts_render_exactly() {
        assert_eq!(ParameterCount::new(999_999).to_string(), "999999");
        assert_eq!(ParameterCount::new(0).to_string(), "0");
    }

    #[test]
    fn the_exact_count_survives_rendering() {
        assert_eq!(ParameterCount::new(8_190_735_360).count(), 8_190_735_360);
    }
}
