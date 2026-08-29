use std::fmt;

/// An exact, non-negative length in bytes of a model artifact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteLength(u64);

impl ByteLength {
    /// A zero-byte artifact; nothing has been received yet.
    pub const ZERO: Self = Self(0);

    /// Builds a length from an exact byte count.
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// The number of bytes this length represents.
    pub const fn bytes(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ByteLength {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.0;
        if bytes < 1_024 {
            write!(formatter, "{bytes} B")
        } else if bytes < 1_048_576 {
            write!(formatter, "{:.1} KiB", bytes as f64 / 1_024.0)
        } else if bytes < 1_073_741_824 {
            write!(formatter, "{:.1} MiB", bytes as f64 / 1_048_576.0)
        } else {
            write!(formatter, "{:.1} GiB", bytes as f64 / 1_073_741_824.0)
        }
    }
}

#[cfg(test)]
mod byte_length_tests {
    use crate::byte_length::ByteLength;

    #[test]
    fn zero_is_the_smallest_length() {
        assert_eq!(ByteLength::ZERO, ByteLength::new(0));
        assert!(ByteLength::ZERO < ByteLength::new(1));
    }

    #[test]
    fn display_switches_units_by_magnitude() {
        assert_eq!(ByteLength::new(512).to_string(), "512 B");
        assert_eq!(ByteLength::new(2048).to_string(), "2.0 KiB");
        assert_eq!(ByteLength::new(5 * 1024 * 1024).to_string(), "5.0 MiB");
        assert_eq!(
            ByteLength::new(2 * 1024 * 1024 * 1024).to_string(),
            "2.0 GiB"
        );
    }
}
