use std::fmt;

use crate::errors::DomainError;

const DIGEST_BYTES: usize = 32;
const HEX_DIGITS: usize = DIGEST_BYTES * 2;

/// A digest that proves the byte integrity of a downloaded model.
///
/// Construction happens either from the exact 32 raw bytes (for adapters that
/// already produced a digest) or from a 64-character hexadecimal literal.
/// Two digests are equal when all 32 bytes match.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Checksum([u8; DIGEST_BYTES]);

impl Checksum {
    /// Builds a digest from its exact 32 raw bytes.
    pub const fn from_bytes(raw: [u8; DIGEST_BYTES]) -> Self {
        Self(raw)
    }

    /// Parses a 64-character hexadecimal literal into a digest.
    ///
    /// Both lowercase and uppercase letters are accepted; anything else is a
    /// `DomainError::InvalidChecksumLiteral`.
    pub fn parse(literal: &str) -> Result<Self, DomainError> {
        if literal.len() != HEX_DIGITS {
            return Err(DomainError::InvalidChecksumLiteral(literal.to_owned()));
        }
        let raw: Vec<u8> = literal
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| decode_hex_pair(pair, literal))
            .collect::<Result<_, _>>()?;
        Ok(Self(raw.try_into().expect("length is checked above")))
    }

    /// Renders the digest as a lowercase hexadecimal string.
    pub fn to_hex(self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

fn decode_hex_pair(pair: &[u8], literal: &str) -> Result<u8, DomainError> {
    let high = decode_hex_digit(char::from(pair[0]), literal)?;
    let low = decode_hex_digit(char::from(pair[1]), literal)?;
    Ok((high << 4) | low)
}

fn decode_hex_digit(character: char, literal: &str) -> Result<u8, DomainError> {
    character
        .to_digit(16)
        .map(|digit| digit as u8)
        .ok_or_else(|| DomainError::InvalidChecksumLiteral(literal.to_owned()))
}

impl fmt::Display for Checksum {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

impl fmt::Debug for Checksum {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

#[cfg(test)]
mod checksum_tests {
    use crate::{errors::DomainError, value_objects::Checksum};

    const SAMPLE_HEX: &str = "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3fdf96d1b0f6a55a0f9f0f7e8";

    #[test]
    fn round_tripping_a_hex_literal_preserves_the_digest() {
        let digest = Checksum::parse(SAMPLE_HEX).expect("sample must parse");
        assert_eq!(digest.to_hex(), SAMPLE_HEX);
        assert_eq!(
            digest,
            Checksum::parse(&digest.to_hex()).expect("hex must re-parse")
        );
    }

    #[test]
    fn uppercase_hex_literals_parse_to_the_same_digest() {
        let lowercase = Checksum::parse(SAMPLE_HEX).expect("lowercase must parse");
        let uppercase = Checksum::parse(&SAMPLE_HEX.to_uppercase()).expect("uppercase must parse");
        assert_eq!(lowercase, uppercase);
    }

    #[test]
    fn reject_literals_that_are_not_64_characters() {
        assert_eq!(
            Checksum::parse("deadbeef"),
            Err(DomainError::InvalidChecksumLiteral("deadbeef".to_owned()))
        );
    }

    #[test]
    fn reject_literals_that_contain_non_hex_characters() {
        assert!(matches!(
            Checksum::parse(&format!("{}z", &SAMPLE_HEX[..63])),
            Err(DomainError::InvalidChecksumLiteral(_))
        ));
    }

    #[test]
    fn from_bytes_round_trips_through_hex() {
        let digest = Checksum::from_bytes([7u8; 32]);
        assert_eq!(digest.to_hex(), "07".repeat(32));
    }

    #[test]
    fn digests_differ_when_a_single_byte_differs() {
        let left = Checksum::from_bytes([0u8; 32]);
        let right = Checksum::from_bytes([1u8; 32]);
        assert_ne!(left, right);
    }
}
