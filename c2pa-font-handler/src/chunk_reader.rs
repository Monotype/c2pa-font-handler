// Copyright 2025 Monotype Imaging Inc.
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

//! Chunk Reader support, used to read chunks of data, getting offset and length
//! information.

use core::fmt;
use std::io::{Read, Seek};

use crate::error::FontIoError;

/// Custom trait for reading chunks of data from a scalable font (SFNT).
pub trait ChunkReader {
    /// The error type for reading the data.
    type Error;
    /// Gets a collection of positions of chunks within the font, used to
    /// omit from hashing.
    fn get_chunk_positions<T: Read + Seek + ?Sized>(
        &self,
        reader: &mut T,
    ) -> core::result::Result<Vec<ChunkPosition>, Self::Error>;
}

/// Identifies types of regions within a font file. Chunks with lesser enum
/// values precede those with greater enum values; order within a given group
/// of chunks (such as a series of `Table` chunks) must be preserved by some
/// other mechanism.
#[derive(Eq, PartialEq)]
pub enum ChunkType {
    /// Whole-container header.
    Header,
    /// Table directory entry or entries.
    _Directory,
    /// Table data included in C2PA hash.
    TableDataIncluded,
    /// Table data excluded from C2PA hash.
    TableDataExcluded,
}

impl std::fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChunkType::Header => write!(f, "Header"),
            ChunkType::_Directory => write!(f, "Directory"),
            ChunkType::TableDataIncluded => write!(f, "TableDataIncluded"),
            ChunkType::TableDataExcluded => write!(f, "TableDataExcluded"),
        }
    }
}

impl std::fmt::Debug for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChunkType::Header => write!(f, "Header"),
            ChunkType::_Directory => write!(f, "Directory"),
            ChunkType::TableDataIncluded => write!(f, "TableDataIncluded"),
            ChunkType::TableDataExcluded => write!(f, "TableDataExcluded"),
        }
    }
}

/// Represents regions within a font file that may be of interest when it
/// comes to hashing data for C2PA.
#[derive(Eq, PartialEq)]
pub struct ChunkPosition {
    /// Offset to the start of the chunk
    pub offset: usize,
    /// Length of the chunk
    pub length: usize,
    /// Tag of the chunk
    pub name: [u8; 4],
    /// Type of chunk
    pub chunk_type: ChunkType,
}

impl ChunkPosition {
    /// Gets the name as an UTF-8 string.
    pub fn name_as_string(&self) -> core::result::Result<String, FontIoError> {
        String::from_utf8(self.name.to_vec())
            .map_err(FontIoError::StringFromUtf8)
    }
}

impl std::fmt::Display for ChunkPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}",
            String::from_utf8_lossy(&self.name),
            self.offset,
            self.length,
            self.chunk_type
        )
    }
}

impl std::fmt::Debug for ChunkPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}, {:?}, {:?}, {:?}",
            String::from_utf8_lossy(&self.name),
            self.offset,
            self.length,
            self.chunk_type
        )
    }
}

#[cfg(test)]
#[path = "chunk_reader_test.rs"]
mod tests;
