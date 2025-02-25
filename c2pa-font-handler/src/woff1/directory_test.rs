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

//! Tests for WOFF1 directory module

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
fn test_woff1_directory_entry_read_exact_with_bad_size() {
    let mut reader = Cursor::new(vec![
        0x74, 0x65, 0x73, 0x74, // tag
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x99, 0x88, 0x77, 0x66, // compLength
        0x12, 0x34, 0x56, 0x78, // origLength
        0x13, 0x57, 0x9b, 0xdf, // origChecksum
    ]);
    let result = Woff1DirectoryEntry::from_reader_exact(&mut reader, 0, 16);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(
        error,
        FontIoError::InvalidSizeForDirectoryEntry {
            expected: 20,
            got: 16
        }
    ));
}

#[test]
fn test_woff1_directory_entry_with_too_small_buffer() {
    // Create data that is purposely too small to read the entry, leaving off
    // the last 4 bytes.
    let entry_data = vec![
        0x74, 0x65, 0x73, 0x74, // tag
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x99, 0x88, 0x77, 0x66, // compLength
        0x12, 0x34, 0x56, 0x78, // origLength
    ];
    let mut reader = Cursor::new(entry_data);
    let result = Woff1DirectoryEntry::from_reader_exact(
        &mut reader,
        0,
        Woff1DirectoryEntry::SIZE,
    );
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, FontIoError::IoError(_)));
}

#[test]
fn test_woff1_directory_entry_write() {
    let entry = Woff1DirectoryEntry {
        tag: FontTag::new(*b"test"),
        offset: 0x9abcdef0,
        compLength: 0x99887766,
        origLength: 0x12345678,
        origChecksum: 0x13579bdf,
    };
    let mut writer = Cursor::new(Vec::new());
    entry.write(&mut writer).unwrap();
    let data = writer.into_inner();
    assert_eq!(
        data,
        vec![
            0x74, 0x65, 0x73, 0x74, // tag
            0x9a, 0xbc, 0xde, 0xf0, // offset
            0x99, 0x88, 0x77, 0x66, // compLength
            0x12, 0x34, 0x56, 0x78, // origLength
            0x13, 0x57, 0x9b, 0xdf, // origChecksum
        ]
    );
}

#[test]
fn test_woff1_directory_entry_write_with_too_small_buffer() {
    let entry = Woff1DirectoryEntry {
        tag: FontTag::new(*b"test"),
        offset: 0x9abcdef0,
        compLength: 0x99887766,
        origLength: 0x12345678,
        origChecksum: 0x13579bdf,
    };
    // Create a fixed size buffer, to simulate a buffer that is too small to
    let buffer: [u8; 8] = [0; 8];
    let mut writer = Cursor::new(buffer);
    let result = entry.write(&mut writer);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, FontIoError::IoError(_)));
}

#[test]
fn test_woff1_directory_entry_checksum() {
    let entry = Woff1DirectoryEntry {
        tag: FontTag::new(*b"test"),
        offset: 0x9abcdef0,
        compLength: 0x99887766,
        origLength: 0x12345678,
        origChecksum: 0x13579bdf,
    };
    let checksum = entry.checksum();
    assert_eq!(
        checksum,
        Wrapping(0x74657374 as u32)
            + Wrapping(0x9abcdef0)
            + Wrapping(0x99887766)
            + Wrapping(0x12345678)
            + Wrapping(0x13579bdf)
    );
}

#[test]
fn test_woff1_directory_add_entry() {
    let mut dir = Woff1Directory::new();
    let entry = Woff1DirectoryEntry {
        tag: FontTag::new(*b"test"),
        offset: 0x9abcdef0,
        compLength: 0x99887766,
        origLength: 0x12345678,
        origChecksum: 0x13579bdf,
    };
    dir.add_entry(entry);
    assert_eq!(dir.entries().len(), 1);
    let entry = dir.entries()[0];
    assert_eq!(entry.tag(), FontTag::new(*b"test"));
    assert_eq!(entry.offset(), 0x9abcdef0);
    assert_eq!(entry.length(), 0x99887766);
    assert_eq!(entry.data_checksum(), 0x13579bdf);
}

#[test]
fn test_woff1_directory_sort_entries() {
    let mut dir = Woff1Directory::new();
    let entry1 = Woff1DirectoryEntry {
        tag: FontTag::new(*b"test"),
        offset: 0x9abcdef0,
        compLength: 0x99887766,
        origLength: 0x12345678,
        origChecksum: 0x13579bdf,
    };
    let entry2 = Woff1DirectoryEntry {
        tag: FontTag::new(*b"abcd"),
        offset: 0x9abcdef0,
        compLength: 0x99887766,
        origLength: 0x12345678,
        origChecksum: 0x13579bdf,
    };
    dir.add_entry(entry1);
    dir.add_entry(entry2);
    dir.sort_entries(|entry| entry.tag());
    assert_eq!(dir.entries().len(), 2);
    assert_eq!(dir.entries()[0].tag(), FontTag::new(*b"abcd"));
    assert_eq!(dir.entries()[1].tag(), FontTag::new(*b"test"));
}

#[test]
fn test_woff1_directory_physical_order() {
    let mut dir = Woff1Directory::new();
    let entry1 = Woff1DirectoryEntry {
        tag: FontTag::new(*b"test"),
        offset: 0x9abcdef0,
        compLength: 0x99887766,
        origLength: 0x12345678,
        origChecksum: 0x13579bdf,
    };
    let entry2 = Woff1DirectoryEntry {
        tag: FontTag::new(*b"abcd"),
        offset: 0x9abcdef0,
        compLength: 0x99887766,
        origLength: 0x12345678,
        origChecksum: 0x13579bdf,
    };
    dir.add_entry(entry1);
    dir.add_entry(entry2);
    let physical_order = dir.physical_order();
    assert_eq!(physical_order.len(), 2);
    assert_eq!(physical_order[0].tag(), FontTag::new(*b"test"));
    assert_eq!(physical_order[1].tag(), FontTag::new(*b"abcd"));
}

#[test]
fn test_woff1_directory_read_exact() {
    let mut reader = Cursor::new(vec![
        0x74, 0x65, 0x73, 0x74, // tag
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x99, 0x88, 0x77, 0x66, // compLength
        0x12, 0x34, 0x56, 0x78, // origLength
        0x13, 0x57, 0x9b, 0xdf, // origChecksum
        0x61, 0x62, 0x63, 0x64, // tag
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x99, 0x88, 0x77, 0x66, // compLength
        0x12, 0x34, 0x56, 0x78, // origLength
        0x13, 0x57, 0x9b, 0xdf, // origChecksum
    ]);
    let result = Woff1Directory::from_reader_exact(&mut reader, 0, 40);
    assert!(result.is_ok());
    let dir = result.unwrap();
    assert_eq!(dir.entries().len(), 2);
    let entry = dir.entries()[0];
    assert_eq!(entry.tag(), FontTag::new(*b"test"));
    assert_eq!(entry.offset(), 0x9abcdef0);
    assert_eq!(entry.length(), 0x99887766);
    assert_eq!(entry.data_checksum(), 0x13579bdf);
    let entry = dir.entries()[1];
    assert_eq!(entry.tag(), FontTag::new(*b"abcd"));
    assert_eq!(entry.offset(), 0x9abcdef0);
    assert_eq!(entry.length(), 0x99887766);
    assert_eq!(entry.data_checksum(), 0x13579bdf);
}

#[test]
fn test_woff1_directory_write() {
    let mut dir = Woff1Directory::new();
    let entry1 = Woff1DirectoryEntry {
        tag: FontTag::new(*b"test"),
        offset: 0x9abcdef0,
        compLength: 0x99887766,
        origLength: 0x12345678,
        origChecksum: 0x13579bdf,
    };
    let entry2 = Woff1DirectoryEntry {
        tag: FontTag::new(*b"abcd"),
        offset: 0x9abcdef0,
        compLength: 0x99887766,
        origLength: 0x12345678,
        origChecksum: 0x13579bdf,
    };
    dir.add_entry(entry1);
    dir.add_entry(entry2);
    let mut writer = Cursor::new(Vec::new());
    dir.write(&mut writer).unwrap();
    let data = writer.into_inner();
    assert_eq!(
        data,
        vec![
            0x74, 0x65, 0x73, 0x74, // tag
            0x9a, 0xbc, 0xde, 0xf0, // offset
            0x99, 0x88, 0x77, 0x66, // compLength
            0x12, 0x34, 0x56, 0x78, // origLength
            0x13, 0x57, 0x9b, 0xdf, // origChecksum
            0x61, 0x62, 0x63, 0x64, // tag
            0x9a, 0xbc, 0xde, 0xf0, // offset
            0x99, 0x88, 0x77, 0x66, // compLength
            0x12, 0x34, 0x56, 0x78, // origLength
            0x13, 0x57, 0x9b, 0xdf, // origChecksum
        ]
    );
}

#[test]
fn test_woff1_directory_checksum() {
    let mut dir = Woff1Directory::new();
    let entry1 = Woff1DirectoryEntry {
        tag: FontTag::new(*b"test"),
        offset: 0x9abcdef0,
        compLength: 0x99887766,
        origLength: 0x12345678,
        origChecksum: 0x13579bdf,
    };
    let entry2 = Woff1DirectoryEntry {
        tag: FontTag::new(*b"abcd"),
        offset: 0x9abcdef0,
        compLength: 0x99887766,
        origLength: 0x12345678,
        origChecksum: 0x13579bdf,
    };
    dir.add_entry(entry1);
    dir.add_entry(entry2);
    let checksum = dir.checksum();
    assert_eq!(
        checksum,
        Wrapping(0x74657374 as u32)
            + Wrapping(0x9abcdef0)
            + Wrapping(0x99887766)
            + Wrapping(0x12345678)
            + Wrapping(0x13579bdf)
            + Wrapping(0x61626364)
            + Wrapping(0x9abcdef0)
            + Wrapping(0x99887766)
            + Wrapping(0x12345678)
            + Wrapping(0x13579bdf)
    );
}

#[test]
fn test_woff1_directory_checksum_zero_entries() {
    let dir = Woff1Directory::new();
    let checksum = dir.checksum();
    assert_eq!(checksum, Wrapping(0));
}

#[test]
fn test_woff1_directory_with_table_count() {
    let mut reader = Cursor::new(vec![
        0x74, 0x65, 0x73, 0x74, // tag
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x99, 0x88, 0x77, 0x66, // compLength
        0x12, 0x34, 0x56, 0x78, // origLength
        0x13, 0x57, 0x9b, 0xdf, // origChecksum
        0x61, 0x62, 0x63, 0x64, // tag
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x99, 0x88, 0x77, 0x66, // compLength
        0x12, 0x34, 0x56, 0x78, // origLength
        0x13, 0x57, 0x9b, 0xdf, // origChecksum
    ]);
    let result = Woff1Directory::from_reader_with_count(&mut reader, 2);
    assert!(result.is_ok());
    let dir = result.unwrap();
    assert_eq!(dir.entries().len(), 2);
    let entry = dir.entries()[0];
    assert_eq!(entry.tag(), FontTag::new(*b"test"));
    assert_eq!(entry.offset(), 0x9abcdef0);
    assert_eq!(entry.length(), 0x99887766);
    assert_eq!(entry.data_checksum(), 0x13579bdf);
    let entry = dir.entries()[1];
    assert_eq!(entry.tag(), FontTag::new(*b"abcd"));
    assert_eq!(entry.offset(), 0x9abcdef0);
    assert_eq!(entry.length(), 0x99887766);
    assert_eq!(entry.data_checksum(), 0x13579bdf);
}

#[test]
fn test_woff1_directory_with_non_4byte_alignment() {
    let mut reader = Cursor::new(vec![
        0x74, 0x65, 0x73, 0x74, // tag
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x99, 0x88, 0x77, 0x66, // compLength
        0x12, 0x34, 0x56, 0x78, // origLength
        0x13, 0x57, 0x9b, 0xdf, // origChecksum
        0x61, 0x62, 0x63, 0x64, // tag
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x99, 0x88, 0x77, 0x66, // compLength
        0x12, 0x34, 0x56, 0x78, // origLength
        0x13, 0x57, 0x9b, 0xdf, // origChecksum
    ]);
    let result = Woff1Directory::from_reader_exact(&mut reader, 0, 21);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, FontIoError::InvalidSizeForDirectory(_)));
}
