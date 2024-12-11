// Copyright 2024 Monotype Imaging Inc.
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

//! Tests for SFNT directory module

use std::io::Cursor;

use super::*;

#[test]
fn test_sfnt_directory_entry_read_exact() {
    let mut reader = Cursor::new(vec![
        0x74, 0x65, 0x73, 0x74, // tag
        0x12, 0x34, 0x56, 0x78, // checksum
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x13, 0x57, 0x9b, 0xdf, // length
    ]);
    let result = SfntDirectoryEntry::from_reader_exact(&mut reader, 0, 16);
    assert!(result.is_ok());
    let entry = result.unwrap();
    assert_eq!(entry.tag, FontTag::new(*b"test"));
    let checksum = entry.checksum;
    assert_eq!(checksum, 0x12345678);
    let offset = entry.offset;
    assert_eq!(offset, 0x9abcdef0);
    let length = entry.length;
    assert_eq!(length, 0x13579bdf);
}

#[test]
fn test_sfnt_directory_entry_read_exact_with_bad_size() {
    let mut reader = Cursor::new(vec![
        0x74, 0x65, 0x73, 0x74, // tag
        0x12, 0x34, 0x56, 0x78, // checksum
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x13, 0x57, 0x9b, 0xdf, // length
    ]);
    let result = SfntDirectoryEntry::from_reader_exact(&mut reader, 0, 15);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(
        err,
        FontIoError::InvalidSizeForDirectoryEntry {
            expected: 16,
            got: 15,
        }
    ));
}

#[test]
fn test_sfnt_directory_entry_with_too_small_buffer() {
    let mut reader = Cursor::new(vec![0; 15]);
    let result = SfntDirectoryEntry::from_reader_exact(&mut reader, 0, 16);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::IoError(_)));
    assert_eq!(err.to_string(), "failed to fill whole buffer");
}

#[test]
fn test_sfnt_directory_entry_write() {
    let entry = SfntDirectoryEntry {
        tag: FontTag::new(*b"test"),
        checksum: 0x12345678,
        offset: 0x9abcdef0,
        length: 0x13579bdf,
    };
    let mut writer = Cursor::new(Vec::new());
    entry.write(&mut writer).unwrap();
    assert_eq!(
        writer.into_inner(),
        vec![
            0x74, 0x65, 0x73, 0x74, // tag
            0x12, 0x34, 0x56, 0x78, // checksum
            0x9a, 0xbc, 0xde, 0xf0, // offset
            0x13, 0x57, 0x9b, 0xdf, // length
        ]
    );
}

#[test]
fn test_sfnt_directory_entry_write_with_too_small_buffer() {
    let entry = SfntDirectoryEntry {
        tag: FontTag::new(*b"test"),
        checksum: 0x12345678,
        offset: 0x9abcdef0,
        length: 0x13579bdf,
    };
    let mut buffer = [0; 15];
    let mut cursor = Cursor::new(&mut buffer[..]);
    let result = entry.write(&mut cursor);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::IoError(_)));
    assert_eq!(err.to_string(), "failed to write whole buffer");
}

#[test]
fn test_sfnt_directory_entry_checksum() {
    let entry = SfntDirectoryEntry {
        tag: FontTag::new(*b"test"),
        checksum: 0x00005678,
        offset: 0x00000003,
        length: 0x00000100,
    };
    let checksum = entry.checksum();
    assert_eq!(
        checksum,
        Wrapping(0x74657374u32 + 0x00005678u32 + 0x00000003u32 + 0x00000100u32)
    );
}

#[test]
fn test_sfnt_directory_add_entry() {
    let mut dir = SfntDirectory::new();
    assert_eq!(dir.entries().len(), 0);

    let entry = SfntDirectoryEntry {
        tag: FontTag::new(*b"test"),
        checksum: 0x12345678,
        offset: 0x9abcdef0,
        length: 0x13579bdf,
    };
    dir.add_entry(entry);
    assert_eq!(dir.entries().len(), 1);
}

#[test]
fn test_sfnt_directory_sort_entries() {
    let mut dir = SfntDirectory::new();
    assert_eq!(dir.entries().len(), 0);

    let entry1 = SfntDirectoryEntry {
        tag: FontTag::new(*b"test"),
        checksum: 0x12345678,
        offset: 0x9abcdef0,
        length: 0x13579bdf,
    };
    let entry2 = SfntDirectoryEntry {
        tag: FontTag::new(*b"best"),
        checksum: 0x12345678,
        offset: 0x9abcdef0,
        length: 0x13579bdf,
    };
    dir.add_entry(entry1);
    dir.add_entry(entry2);
    assert_eq!(dir.entries().len(), 2);
    assert_eq!(dir.entries()[0].tag, FontTag::new(*b"test"));
    assert_eq!(dir.entries()[1].tag, FontTag::new(*b"best"));

    dir.sort_entries(|entry| entry.tag);
    assert_eq!(dir.entries().len(), 2);
    assert_eq!(dir.entries()[0].tag, FontTag::new(*b"best"));
    assert_eq!(dir.entries()[1].tag, FontTag::new(*b"test"));
}

#[test]
fn test_sfnt_directory_physical_order() {
    let mut dir = SfntDirectory::new();
    assert_eq!(dir.entries().len(), 0);

    let entry1 = SfntDirectoryEntry {
        tag: FontTag::new(*b"test"),
        checksum: 0x12345678,
        offset: 0x9abcdef0,
        length: 0x13579bdf,
    };
    let entry2 = SfntDirectoryEntry {
        tag: FontTag::new(*b"best"),
        checksum: 0x12345678,
        offset: 0x9abcdef0,
        length: 0x13579bdf,
    };
    dir.add_entry(entry1);
    dir.add_entry(entry2);
    assert_eq!(dir.entries().len(), 2);
    assert_eq!(dir.entries()[0].tag, FontTag::new(*b"test"));
    assert_eq!(dir.entries()[1].tag, FontTag::new(*b"best"));

    let physical_order = dir.physical_order();
    assert_eq!(physical_order.len(), 2);
    assert_eq!(physical_order[0].tag, FontTag::new(*b"test"));
    assert_eq!(physical_order[1].tag, FontTag::new(*b"best"));
}

#[test]
fn test_sfnt_directory_read_exact() {
    let mut reader = Cursor::new(vec![
        0x74, 0x65, 0x73, 0x74, // tag
        0x12, 0x34, 0x56, 0x78, // checksum
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x13, 0x57, 0x9b, 0xdf, // length
    ]);
    let result = SfntDirectory::from_reader_exact(&mut reader, 0, 16);
    assert!(result.is_ok());
    let dir = result.unwrap();
    assert_eq!(dir.entries().len(), 1);
    let entry = &dir.entries()[0];
    assert_eq!(entry.tag, FontTag::new(*b"test"));
    let checksum = entry.checksum;
    assert_eq!(checksum, 0x12345678);
    let offset = entry.offset;
    assert_eq!(offset, 0x9abcdef0);
    let length = entry.length;
    assert_eq!(length, 0x13579bdf);
}

#[test]
fn test_sfnt_directory_read_exact_multiple_tables() {
    let mut reader = Cursor::new(vec![
        0x74, 0x65, 0x73, 0x74, // tag
        0x12, 0x34, 0x56, 0x78, // checksum
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x13, 0x57, 0x9b, 0xdf, // length
        0x62, 0x65, 0x73, 0x74, // tag
        0x12, 0x34, 0x56, 0x78, // checksum
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x13, 0x57, 0x9b, 0xdf, // length
    ]);
    let result = SfntDirectory::from_reader_exact(&mut reader, 0, 32);
    assert!(result.is_ok());
    let dir = result.unwrap();
    assert_eq!(dir.entries().len(), 2);
    let entry1 = &dir.entries()[0];
    assert_eq!(entry1.tag, FontTag::new(*b"test"));
    let checksum1 = entry1.checksum;
    assert_eq!(checksum1, 0x12345678);
    let offset1 = entry1.offset;
    assert_eq!(offset1, 0x9abcdef0);
    let length1 = entry1.length;
    assert_eq!(length1, 0x13579bdf);
    let entry2 = &dir.entries()[1];
    assert_eq!(entry2.tag, FontTag::new(*b"best"));
    let checksum2 = entry2.checksum;
    assert_eq!(checksum2, 0x12345678);
    let offset2 = entry2.offset;
    assert_eq!(offset2, 0x9abcdef0);
    let length2 = entry2.length;
    assert_eq!(length2, 0x13579bdf);
}

#[test]
fn test_sfnt_directory_write() {
    let mut dir = SfntDirectory::new();
    let entry1 = SfntDirectoryEntry {
        tag: FontTag::new(*b"test"),
        checksum: 0x12345678,
        offset: 0x9abcdef0,
        length: 0x13579bdf,
    };
    let entry2 = SfntDirectoryEntry {
        tag: FontTag::new(*b"best"),
        checksum: 0x12345678,
        offset: 0x9abcdef0,
        length: 0x13579bdf,
    };
    dir.add_entry(entry1);
    dir.add_entry(entry2);
    let mut buffer = Vec::new();
    dir.write(&mut buffer).unwrap();
    assert_eq!(
        buffer,
        vec![
            0x74, 0x65, 0x73, 0x74, // tag
            0x12, 0x34, 0x56, 0x78, // checksum
            0x9a, 0xbc, 0xde, 0xf0, // offset
            0x13, 0x57, 0x9b, 0xdf, // length
            0x62, 0x65, 0x73, 0x74, // tag
            0x12, 0x34, 0x56, 0x78, // checksum
            0x9a, 0xbc, 0xde, 0xf0, // offset
            0x13, 0x57, 0x9b, 0xdf, // length
        ]
    );
}

#[test]
fn test_sfnt_directory_checksum() {
    let mut dir = SfntDirectory::new();
    let entry1 = SfntDirectoryEntry {
        tag: FontTag::new(*b"test"),
        checksum: 0x00005678,
        offset: 0x00000003,
        length: 0x00000100,
    };
    let entry2 = SfntDirectoryEntry {
        tag: FontTag::new(*b"best"),
        checksum: 0x00005678,
        offset: 0x00000003,
        length: 0x00000100,
    };
    dir.add_entry(entry1);
    dir.add_entry(entry2);
    let checksum = dir.checksum();
    assert_eq!(
        checksum,
        Wrapping(0x74657374u32 + 0x00005678u32 + 0x00000003u32 + 0x00000100u32)
            + Wrapping(
                0x62657374u32 + 0x00005678u32 + 0x00000003u32 + 0x00000100u32
            )
    );
}

#[test]
fn test_sfnt_directory_checksum_zero_entries() {
    let dir = SfntDirectory::new();
    let checksum = dir.checksum();
    assert_eq!(checksum, Wrapping(0));
}

#[test]
fn test_sfnt_directory_read_exact_without_4byte_aligned() {
    let mut reader = Cursor::new(vec![
        0x74, 0x65, 0x73, 0x74, // tag
        0x12, 0x34, 0x56, 0x78, // checksum
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x13, 0x57, 0x9b, 0xdf, // length
        0x00, 0x00, 0x00, 0x00, // padding
    ]);
    let result = SfntDirectory::from_reader_exact(&mut reader, 0, 20);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::InvalidSizeForDirectory(20)));
}

#[test]
fn test_sfnt_directory_with_table_count() {
    let mut reader = Cursor::new(vec![
        0x74, 0x65, 0x73, 0x74, // tag
        0x12, 0x34, 0x56, 0x78, // checksum
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x13, 0x57, 0x9b, 0xdf, // length
        0x62, 0x65, 0x73, 0x74, // tag
        0x12, 0x34, 0x56, 0x78, // checksum
        0x9a, 0xbc, 0xde, 0xf0, // offset
        0x13, 0x57, 0x9b, 0xdf, // length
    ]);
    let result = SfntDirectory::from_reader_with_count(&mut reader, 2);
    assert!(result.is_ok());
    let dir = result.unwrap();
    assert_eq!(dir.entries().len(), 2);
    let entry1 = &dir.entries()[0];
    assert_eq!(entry1.tag, FontTag::new(*b"test"));
    let checksum1 = entry1.checksum;
    assert_eq!(checksum1, 0x12345678);
    let offset1 = entry1.offset;
    assert_eq!(offset1, 0x9abcdef0);
    let length1 = entry1.length;
    assert_eq!(length1, 0x13579bdf);
    let entry2 = &dir.entries()[1];
    assert_eq!(entry2.tag, FontTag::new(*b"best"));
    let checksum2 = entry2.checksum;
    assert_eq!(checksum2, 0x12345678);
    let offset2 = entry2.offset;
    assert_eq!(offset2, 0x9abcdef0);
    let length2 = entry2.length;
    assert_eq!(length2, 0x13579bdf);
}
