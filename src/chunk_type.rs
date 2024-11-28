#![allow(unused)]
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ChunkTypeError {
    #[error("Only ASCII characters can be used.")]
    NonAsciiCharFound,
    #[error("Length of a chunk must be 4 bytes.")]
    InvalidLength,
    #[error("Need to know why??")]
    SomeOtherThing,
}

// DAMM: A Rust String is just a Vec<u8> whose bytes have been validated as UTF-8 ~ [pngme book]

#[derive(Debug, PartialEq)]
pub struct ChunkType {
    // u8 because ascii
    pub bytes: [u8; 4],
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.bytes
    }
    /// Checks Whether the all 4 bytes is valid char or not
    pub fn is_valid_byte(&self) -> bool {
        self.bytes().iter().all(|x| x.is_ascii_alphabetic())
    }

    // Should have read documentation clearly, it specifically said about the reserved bit part but ig im dumb
    /// Should always return true, if not, consider it a bug.
    pub fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid() && self.is_valid_byte()
    }

    /// Checks if `this` chunk is necessary to display the PNG
    pub fn is_critical(&self) -> bool {
        self.bytes()
            .first()
            .expect("This should not have happened. Report the bug.")
            .is_ascii_uppercase()
    }

    /// ## Not part of public API.
    /// Not even sure what this is for.
    fn is_public(&self) -> bool {
        self.bytes()
            .get(1)
            .expect("This should not have happened. Report the bug.")
            .is_ascii_uppercase()
    }

    /// Reserved for future iterations of PNG spec.
    pub fn is_reserved_bit_valid(&self) -> bool {
        self.bytes()
            .get(2)
            .expect("This should not have happened. Report the bug.")
            .is_ascii_uppercase()
    }

    /// Irrelevant for decoders but useful in img editors tells whether
    /// the chunk is okay to be copied for the modified version of the img
    pub fn is_safe_to_copy(&self) -> bool {
        self.bytes()
            .get(3)
            .expect("This should not have happened. Report the bug.")
            .is_ascii_lowercase()
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = ChunkTypeError;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        Ok(Self { bytes: value })
    }
}

impl FromStr for ChunkType {
    type Err = ChunkTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(ChunkTypeError::InvalidLength);
        }

        let bytes = s
            .as_bytes()
            .try_into()
            .map_err(|_| ChunkTypeError::SomeOtherThing)?;

        let chunk = Self { bytes };

        if !chunk.is_valid_byte() {
            return Err(ChunkTypeError::NonAsciiCharFound);
        }

        Ok(ChunkType { bytes })
    }
}

// Should this display UTF-8 or raw bytes?
impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{}", std::str::from_utf8(&self.bytes).unwrap())
        write!(
            f,
            "{}",
            self.bytes().map(char::from).iter().collect::<String>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    // Test that a chunk with non-ASCII characters returns the NonAsciiCharFound error
    #[test]
    fn test_non_ascii_char_found() {
        // it actually takes 2 bytes: é. ascii man.
        let result = ChunkType::from_str("bcé");
        assert_eq!(result.unwrap_err(), ChunkTypeError::NonAsciiCharFound);
    }

    // Test that a chunk with an invalid length returns the InvalidLength error
    #[test]
    fn test_invalid_length() {
        let result = ChunkType::from_str("abc"); // Length is less than 4
        assert_eq!(result.unwrap_err(), ChunkTypeError::InvalidLength);
    }

    // Test that a chunk that causes a mapping error returns SomeOtherThing error
    #[test]
    fn test_some_other_thing_error() {
        let result = ChunkType::from_str("abcdxyz"); // Too many characters
        dbg!(&result);

        assert_eq!(result.unwrap_err(), ChunkTypeError::InvalidLength);
    }

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();

        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        println!("kya aa rha re {} ", &chunk.to_string());
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
