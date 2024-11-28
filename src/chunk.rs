// Getters are for API stability

#![allow(unused)]
use crate::chunk_type::ChunkType;
use std::{fmt::Display, io::Read, str::FromStr, string::FromUtf8Error};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ChunkError {
    #[error("Data in this chunk is not valid UTF8 characters.")]
    InvalidUtf8(#[from] FromUtf8Error),
}

pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

impl Chunk {
    fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        // CRC is calculated over everything except length
        let crc = {
            use crc::{Crc, CRC_32_ISO_HDLC};
            const CRC: crc::Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
            let crc_bytes = [chunk_type.bytes().as_ref(), &data].concat();

            CRC.checksum(&crc_bytes)
        };

        Self {
            length: data.len() as u32,
            chunk_type,
            data,
            crc,
        }
    }

    fn length(&self) -> u32 {
        self.length
    }

    fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn crc(&self) -> u32 {
        self.crc
    }

    /// Returns the data stored in this chunk as a `String`. This function will return an error
    /// if the stored data is not valid UTF-8.
    fn data_as_string(&self) -> Result<String, ChunkError> {
        // although i have used attr macro to convert this error, i find Ok(syntax?) rather ugly
        String::from_utf8(self.data().to_vec()).map_err(ChunkError::InvalidUtf8)
    }

    /// Returns this chunk as a byte sequences described by the PNG spec.
    /// The following data is included in this byte sequence in order:
    /// 1. Length of the data *(4 bytes)*
    /// 2. Chunk type *(4 bytes)*
    /// 3. The data itself *(`length` bytes)*
    /// 4. The CRC of the chunk type and data *(4 bytes)*
    fn as_bytes(&self) -> Vec<u8> {
        self.length()
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type().bytes().iter())
            .chain(self.data().iter())
            .chain(self.crc().to_be_bytes().iter())
            .copied()
            .collect()
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
