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

//! Tests for the chunk reader types

use super::*;

#[test]
fn test_chunk_position() {
    let chunk = ChunkPosition::new(0, 4, *b"head", ChunkType::Header, true);

    assert_eq!(chunk.offset(), 0);
    assert_eq!(chunk.length(), 4);
    assert_eq!(chunk.name(), b"head");
    let name_result = chunk.name_as_string();
    assert!(name_result.is_ok());
    assert_eq!(name_result.unwrap(), "head");
    assert_eq!(chunk.chunk_type(), &ChunkType::Header);
    assert_eq!(chunk.should_hash(), true);
}

#[test]
fn test_chunk_position_display() {
    let chunk = ChunkPosition {
        offset: 0,
        length: 4,
        name: *b"head",
        chunk_type: ChunkType::Header,
        should_hash: true,
    };

    assert_eq!(
        chunk.to_string(),
        "Chunk(Header): head at offset 0 with length 4; hash: true"
    );
}

#[test]
fn test_chunk_type_display() {
    assert_eq!(ChunkType::Header.to_string(), "Header");
    assert_eq!(ChunkType::DirectoryEntry.to_string(), "Directory Entry");
    assert_eq!(ChunkType::TableData.to_string(), "Table Data");
}
