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

//! Tests for chunk reader

use super::*;

#[test]
fn test_chunk_type_display_fmt() {
    assert_eq!(format!("{}", ChunkType::Header), "Header");
    assert_eq!(format!("{}", ChunkType::_Directory), "Directory");
    assert_eq!(
        format!("{}", ChunkType::TableDataIncluded),
        "TableDataIncluded"
    );
    assert_eq!(
        format!("{}", ChunkType::TableDataExcluded),
        "TableDataExcluded"
    );
}

#[test]
fn test_chunk_type_debug_fmt() {
    assert_eq!(format!("{:?}", ChunkType::Header), "Header");
    assert_eq!(format!("{:?}", ChunkType::_Directory), "Directory");
    assert_eq!(
        format!("{:?}", ChunkType::TableDataIncluded),
        "TableDataIncluded"
    );
    assert_eq!(
        format!("{:?}", ChunkType::TableDataExcluded),
        "TableDataExcluded"
    );
}

#[test]
fn test_chunk_position_name_as_str() {
    let chunk_position = ChunkPosition {
        chunk_type: ChunkType::Header,
        offset: 0,
        name: [b'H', b'e', b'a', b'd'],
        length: 12,
    };
    let result = chunk_position.name_as_string();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Head");
}

#[test]
fn test_chunk_position_display_fmt() {
    let chunk_position = ChunkPosition {
        chunk_type: ChunkType::Header,
        offset: 0,
        name: [b'H', b'e', b'a', b'd'],
        length: 12,
    };
    assert_eq!(format!("{}", chunk_position), "Head, 0, 12, Header");
}

#[test]
fn test_chunk_position_debug_fmt() {
    let chunk_position = ChunkPosition {
        chunk_type: ChunkType::Header,
        offset: 0,
        name: [b'H', b'e', b'a', b'd'],
        length: 12,
    };
    assert_eq!(format!("{:?}", chunk_position), "\"Head\", 0, 12, Header");
}
