// Getters are for API stability

use std::fmt::Display;
use std::string::FromUtf8Error;

use thiserror::Error;

use crate::chunk_type::{ChunkType, ChunkTypeError};

#[derive(Debug, Error)]
pub enum ChunkError {
	#[error("Data in this chunk is not valid UTF8 characters.")]
	InvalidUtf8(#[from] FromUtf8Error),
	#[error("Chunk length must be equal to {0}")]
	ShortInput(usize),
	#[error("Unable to convert slice. {0}")]
	SliceToSized(#[from] std::array::TryFromSliceError),
	#[error("{0}")]
	ChunkTypeError(#[from] ChunkTypeError),
	#[error("CRC doesnt match! found: {found_crc}, expected: {expected_crc}")]
	IncorrectCrc { found_crc: u32, expected_crc: u32 },
}

#[derive(Debug)]
pub struct Chunk {
	chunk_type: ChunkType,
	data: Vec<u8>,
}

impl Chunk {
	pub const CHUNK_TYPE_BYTES: usize = 4;
	// data can't be const cause thats the data hahah ;(
	pub const CRC_LENGTH_BYTES: usize = 4;
	pub const LENGTH_BYTES: usize = 4;
	pub const METADATA_BYTES: usize =
		Self::LENGTH_BYTES + Self::CHUNK_TYPE_BYTES + Self::CRC_LENGTH_BYTES;

	pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
		// CRC is calculated over everything except length

		Self {
			chunk_type,
			data,
			// crc,
		}
	}

	pub fn length(&self) -> u32 {
		self.data().len() as u32
	}

	pub fn chunk_type(&self) -> &ChunkType {
		&self.chunk_type
	}

	pub fn data(&self) -> &[u8] {
		&self.data
	}

	/// Calculating the crc
	pub fn crc(&self) -> u32 {
		use crc::{Crc, CRC_32_ISO_HDLC};
		const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

		// this is different from self.as_bytes as it doesn't include crc, length
		let bytes: Vec<u8> = self
			.chunk_type()
			.bytes()
			.iter()
			.chain(self.data.iter())
			.copied()
			.collect();

		CRC.checksum(&bytes)
	}

	/// Returns the data stored in this chunk as a `String`. This function will
	/// return an error if the stored data is not valid UTF-8.
	pub fn data_as_string(&self) -> Result<String, ChunkError> {
		// although i have used attr macro to convert this error, i find Ok(syntax?)
		// rather ugly
		String::from_utf8(self.data().to_vec()).map_err(ChunkError::InvalidUtf8)
	}

	/// Returns this chunk as a byte sequences described by the PNG spec.
	/// The following data is included in this byte sequence in order:
	/// 1. Length of the data *(4 bytes)*
	/// 2. Chunk type *(4 bytes)*
	/// 3. The data itself *(`length` bytes)*
	// I spent hours debugging because i was operating on usize and not u32.
	// Damm, systems programming aint direct
	pub fn as_bytes(&self) -> Vec<u8> {
		let data_len = self.data().len() as u32;
		data_len
			.to_be_bytes()
			.iter()
			.chain(self.chunk_type.bytes().iter())
			.chain(self.data().iter())
			.chain(self.crc().to_be_bytes().iter())
			.copied()
			.collect()
	}
}

impl TryFrom<&[u8]> for Chunk {
	type Error = ChunkError;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		if value.len() < Self::METADATA_BYTES {
			return Err(ChunkError::ShortInput(Self::METADATA_BYTES));
		}

		// consume first 4 bytes as data length
		let (data_length, value) = value.split_at(Chunk::LENGTH_BYTES);
		let data_length = u32::from_be_bytes(data_length.try_into()?);

		let (chunk_type_bytes, value) = value.split_at(Self::CHUNK_TYPE_BYTES);
		let chunk_type_bytes: [u8; 4] = chunk_type_bytes.try_into()?;
		let chunk_type: ChunkType = chunk_type_bytes.try_into()?;
		chunk_type.is_valid()?;

		// because crc is calculated on preceeding bytes
		let (data, value) = value.split_at(data_length as usize);
		let (crc_bytes, _) = value.split_at(Self::CRC_LENGTH_BYTES);

		let tmp = Self {
			chunk_type,
			data: data.into(),
		};

		let found_crc = u32::from_be_bytes(crc_bytes.try_into()?);
		let expected_crc = tmp.crc();

		if found_crc != expected_crc {
			return Err(ChunkError::IncorrectCrc {
				found_crc,
				expected_crc,
			});
		}

		Ok(Self {
			chunk_type,
			data: data.into(),
		})
	}
}

impl Display for Chunk {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}: {:?}", self.chunk_type(), self.data())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::chunk_type::ChunkType;
	use std::str::FromStr;

	fn testing_chunk() -> Chunk {
		let data_length: u32 = 42;
		let chunk_type = "RuSt".as_bytes();
		let message_bytes = "This is where your secret message will be!".as_bytes();
		let crc: u32 = 2882656334;

		let chunk_data: Vec<u8> = data_length
			.to_be_bytes()
			.iter()
			.chain(chunk_type.iter())
			.chain(message_bytes.iter())
			.chain(crc.to_be_bytes().iter())
			.copied()
			.collect();

		Chunk::try_from(chunk_data.as_ref()).unwrap()
	}

	#[test]
	fn test_new_chunk() {
		let chunk_type = ChunkType::from_str("RuSt").unwrap();
		let data = "This is where your secret message will be!"
			.as_bytes()
			.to_vec();
		let chunk = Chunk::new(chunk_type, data);
		assert_eq!(chunk.length(), 42);
		assert_eq!(chunk.crc(), 2882656334);
	}

	#[test]
	fn test_chunk_length() {
		let chunk = testing_chunk();
		assert_eq!(chunk.length(), 42);
	}

	#[test]
	fn test_chunk_type() {
		let chunk = testing_chunk();
		assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
	}

	#[test]
	fn test_chunk_string() {
		let chunk = testing_chunk();
		let chunk_string = chunk.data_as_string().unwrap();
		let expected_chunk_string = String::from("This is where your secret message will be!");
		assert_eq!(chunk_string, expected_chunk_string);
	}

	#[test]
	fn test_chunk_crc() {
		let chunk = testing_chunk();
		assert_eq!(chunk.crc(), 2882656334);
	}

	#[test]
	fn test_valid_chunk_from_bytes() {
		let data_length: u32 = 42;
		let chunk_type = "RuSt".as_bytes();
		let message_bytes = "This is where your secret message will be!".as_bytes();
		let crc: u32 = 2882656334;

		let chunk_data: Vec<u8> = data_length
			.to_be_bytes()
			.iter()
			.chain(chunk_type.iter())
			.chain(message_bytes.iter())
			.chain(crc.to_be_bytes().iter())
			.copied()
			.collect();

		let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

		let chunk_string = chunk.data_as_string().unwrap();
		let expected_chunk_string = String::from("This is where your secret message will be!");

		assert_eq!(chunk.length(), 42);
		assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
		assert_eq!(chunk_string, expected_chunk_string);
		assert_eq!(chunk.crc(), 2882656334);
	}

	#[test]
	fn test_invalid_chunk_from_bytes() {
		let data_length: u32 = 42;
		let chunk_type = "RuSt".as_bytes();
		let message_bytes = "This is where your secret message will be!".as_bytes();
		let crc: u32 = 2882656333;

		let chunk_data: Vec<u8> = data_length
			.to_be_bytes()
			.iter()
			.chain(chunk_type.iter())
			.chain(message_bytes.iter())
			.chain(crc.to_be_bytes().iter())
			.copied()
			.collect();

		let chunk = Chunk::try_from(chunk_data.as_ref());

		assert!(chunk.is_err());
	}

	#[test]
	pub fn test_chunk_trait_impls() {
		let data_length: u32 = 42;
		let chunk_type = "RuSt".as_bytes();
		let message_bytes = "This is where your secret message will be!".as_bytes();
		let crc: u32 = 2882656334;

		let chunk_data: Vec<u8> = data_length
			.to_be_bytes()
			.iter()
			.chain(chunk_type.iter())
			.chain(message_bytes.iter())
			.chain(crc.to_be_bytes().iter())
			.copied()
			.collect();

		let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();
		println!("{}", chunk);

		let _chunk_string = format!("{}", chunk);
	}
}
