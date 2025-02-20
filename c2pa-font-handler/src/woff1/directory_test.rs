// Copyright 2024-2025 Monotype Imaging Inc.
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

//! Tests for woff1 directory module

use std::io::Cursor;

use super::*;

#[test]
fn test_woff1_directory_entry_read_exact() {
    let mut reader = Cursor::new(vec![
        0x74, 0x65, 0x73, 0x74, // tag
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x99, 0x88, 0x77, 0x66, // compLength
        0x12, 0x34, 0x56, 0x78, // origLength
        0x13, 0x57, 0x9b, 0xdf, // origChecksum
    ]);
    let result = Woff1DirectoryEntry::from_reader_exact(&mut reader, 0, 20);
    assert!(result.is_ok());
    let entry = result.unwrap();
    assert_eq!(entry.tag(), FontTag::new(*b"test"));
    assert_eq!(entry.offset(), 0x9abcdef0);
    assert_eq!(entry.length(), 0x99887766);
    assert_eq!(entry.data_checksum(), 0x13579bdf);
}

#[test]
fn test_woff1_directory_entry_read_exact_with_bad_size() {}

#[test]
fn test_woff1_directory_entry_with_too_small_buffer() {}

#[test]
fn test_woff1_directory_entry_write() {}

#[test]
fn test_woff1_directory_entry_write_with_too_small_buffer() {}

#[test]
fn test_woff1_directory_entry_checksum() {}

#[test]
fn test_woff1_directory_add_entry() {}

#[test]
fn test_woff1_directory_sort_entries() {}

#[test]
fn test_woff1_directory_physical_order() {}

#[test]
fn test_woff1_directory_read_exact() {}

#[test]
fn test_woff1_directory_read_exact_multiple_tables() {}

#[test]
fn test_woff1_directory_write() {}

#[test]
fn test_woff1_directory_checksum() {}

#[test]
fn test_woff1_directory_checksum_zero_entries() {
    let dir = Woff1Directory::new();
    let checksum = dir.checksum();
    assert_eq!(checksum, Wrapping(0));
}

#[test]
fn test_woff1_directory_read_exact_without_4byte_aligned() {}

#[test]
fn test_woff1_directory_with_table_count() {}
