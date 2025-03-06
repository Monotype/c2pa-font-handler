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

//! Definitions of chunks for various font softwares

use std::{
    fmt::Display,
    io::{Read, Seek},
};

/// A trait for reading data chunks.
pub trait ChunkReader {
    /// The error type for reading data chunks.
    type Error;
    /// The type of chunk.
    type ChunkType: ChunkTypeTrait;

    /// Get the positions of all chunks in the data.
    fn get_chunk_positions(
        reader: &mut (impl Read + Seek + ?Sized),
    ) -> Result<Vec<ChunkPosition<Self::ChunkType>>, Self::Error>;
}

/// Defines a chunk type
pub trait ChunkTypeTrait:
    Clone + std::fmt::Debug + Eq + PartialEq + Display
{
    /// Whether the chunk should be hashed
    ///
    /// # Remarks
    /// The default is to hash the chunk
    fn should_hash(&self) -> bool {
        true
    }
}

/// A chunk position
#[derive(Debug, Eq, PartialEq)]
pub struct ChunkPosition<T: ChunkTypeTrait> {
    /// Offset to the start of the chunk
    offset: usize,
    /// Length of the chunk
    length: usize,
    /// Name, or tag, of the chunk
    name: [u8; 4],
    /// Type of chunk
    chunk_type: T,
}

impl<T: ChunkTypeTrait> ChunkPosition<T> {
    /// Create a new chunk position
    pub fn new(
        offset: usize,
        length: usize,
        name: [u8; 4],
        chunk_type: T,
    ) -> Self {
        Self {
            offset,
            length,
            name,
            chunk_type,
        }
    }

    /// Get the name as a string
    pub fn name_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.name.to_vec())
    }

    /// Get the offset of the chunk
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get the length of the chunk data
    pub fn length(&self) -> usize {
        self.length
    }

    /// Get the name of the chunk
    pub fn name(&self) -> &[u8; 4] {
        &self.name
    }

    /// Get the type of the chunk
    pub fn chunk_type(&self) -> &T {
        &self.chunk_type
    }
}

impl<T: ChunkTypeTrait> std::fmt::Display for ChunkPosition<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Chunk({}): {} at offset {} with length {}",
            self.chunk_type,
            String::from_utf8_lossy(&self.name),
            self.offset,
            self.length,
        )
    }
}

#[cfg(test)]
#[path = "chunks_test.rs"]
mod tests;
