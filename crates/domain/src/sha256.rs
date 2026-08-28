use std::fmt;

use crate::domain_error::DomainError;

const DIGEST_BYTES: usize = 32;
const HEX_DIGITS: usize = DIGEST_BYTES * 2;

/// A SHA-256 digest that proves the byte integrity of a downloaded model.
///
/// Construction happens either from the exact 32 raw bytes (for adapters that
/// already produced a digest) or from a 64-character hexadecimal literal.
/// Two digests are equal when all 32 bytes match.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sha256([u8; DIGEST_BYTES]);

impl Sha256 {
    /// Builds a digest from its exact 32 raw bytes.
    pub const fn from_bytes(raw: [u8; DIGEST_BYTES]) -> Self {
        Self(raw)
    }

    /// Parses a 64-character hexadecimal literal into a digest.
    ///
    /// Both lowercase and uppercase letters are accepted; anything else is a
    /// `DomainError::InvalidSha256Literal`.
    pub fn parse(literal: &str) -> Result<Self, DomainError> {
        if literal.len() != HEX_DIGITS {
            return Err(DomainError::InvalidSha256Literal(literal.to_owned()));
        }
        let mut raw = [0u8; DIGEST_BYTES];
        let characters: Vec<char> = literal.chars().collect();
        for (index, byte) in raw.iter_mut().enumerate() {
            let high = decode_hex_digit(characters[index * 2], literal)?;
            let low = decode_hex_digit(characters[index * 2 + 1], literal)?;
            *byte = (high << 4) | low;
        }
        Ok(Self(raw))
    }

    /// Renders the digest as a lowercase hexadecimal string.
    pub fn to_hex(self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    /// Borrows the 32 raw bytes of the digest.
    pub fn as_bytes(&self) -> &[u8; DIGEST_BYTES] {
        &self.0
    }

    /// Consumes the digest and returns its 32 raw bytes.
    pub fn into_bytes(self) -> [u8; DIGEST_BYTES] {
        self.0
    }
}

fn decode_hex_digit(character: char, literal: &str) -> Result<u8, DomainError> {
    character
        .to_digit(16)
        .map(|digit| digit as u8)
        .ok_or_else(|| DomainError::InvalidSha256Literal(literal.to_owned()))
}

impl fmt::Display for Sha256 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

impl fmt::Debug for Sha256 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

#[cfg(test)]
mod sha256_tests {
    use crate::domain_error::DomainError;
    use crate::sha256::Sha256;

    const SAMPLE_HEX: &str = "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3fdf96d1b0f6a55a0f9f0f7e8";

    #[test]
    fn round_tripping_a_hex_literal_preserves_the_digest() {
        let digest = Sha256::parse(SAMPLE_HEX).expect("sample must parse");
        assert_eq!(digest.to_hex(), SAMPLE_HEX);
        assert_eq!(
            digest,
            Sha256::parse(&digest.to_hex()).expect("hex must re-parse")
        );
    }

    #[test]
    fn uppercase_hex_literals_parse_to_the_same_digest() {
        let lowercase = Sha256::parse(SAMPLE_HEX).expect("lowercase must parse");
        let uppercase = Sha256::parse(&SAMPLE_HEX.to_uppercase()).expect("uppercase must parse");
        assert_eq!(lowercase, uppercase);
    }

    #[test]
    fn reject_literals_that_are_not_64_characters() {
        assert_eq!(
            Sha256::parse("deadbeef"),
            Err(DomainError::InvalidSha256Literal("deadbeef".to_owned()))
        );
    }

    #[test]
    fn reject_literals_that_contain_non_hex_characters() {
        assert!(matches!(
            Sha256::parse(&format!("{}z", &SAMPLE_HEX[..63])),
            Err(DomainError::InvalidSha256Literal(_))
        ));
    }

    #[test]
    fn from_bytes_round_trips_through_hex() {
        let digest = Sha256::from_bytes([7u8; 32]);
        assert_eq!(digest.to_hex(), "07".repeat(32));
        assert_eq!(digest.into_bytes(), [7u8; 32]);
    }

    #[test]
    fn digests_differ_when_a_single_byte_differs() {
        let left = Sha256::from_bytes([0u8; 32]);
        let right = Sha256::from_bytes([1u8; 32]);
        assert_ne!(left, right);
    }
}
