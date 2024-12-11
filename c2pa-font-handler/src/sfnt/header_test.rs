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

//! Tests for SFNT header module

use std::io::Cursor;

use super::*;

#[test]
fn test_sfnt_header_default() {
    let header = SfntHeader::default();
    assert_eq!(header.sfntVersion as u32, 0x00010000);
    assert_eq!({ header.numTables }, 0);
    assert_eq!({ header.searchRange }, 0);
    assert_eq!({ header.entrySelector }, 0);
    assert_eq!({ header.rangeShift }, 0);
}

#[test]
fn test_sfnt_header_read_exact() {
    let mut reader = Cursor::new(vec![
        0x00, 0x01, 0x00, 0x00, // sfntVersion
        0x00, 0x00, // numTables
        0x00, 0x00, // searchRange
        0x00, 0x00, // entrySelector
        0x00, 0x00, // rangeShift
    ]);
    let result = SfntHeader::from_reader_exact(&mut reader, 0, 12);
    assert!(result.is_ok());
    let header = result.unwrap();
    assert_eq!(header.sfntVersion as u32, 0x00010000);
    assert_eq!({ header.numTables }, 0);
    assert_eq!({ header.searchRange }, 0);
    assert_eq!({ header.entrySelector }, 0);
    assert_eq!({ header.rangeShift }, 0);
}

#[test]
fn test_sfnt_header_read_exact_with_bad_size() {
    let mut reader = Cursor::new(vec![
        0x00, 0x01, 0x00, 0x00, // sfntVersion
        0x00, 0x00, // numTables
        0x00, 0x00, // searchRange
        0x00, 0x00, // entrySelector
        0x00, 0x00, // rangeShift
    ]);
    let result = SfntHeader::from_reader_exact(&mut reader, 0, 11);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::InvalidSizeForHeader(11)));
}

#[test]
fn test_sfnt_header_read_exact_too_small_buffer() {
    let mut reader = Cursor::new(vec![
        0x00, 0x01, 0x00, 0x00, // sfntVersion
        0x00, 0x00, // numTables
        0x00, 0x00, // searchRange
        0x00,
        0x00, /* entrySelector
               * missing rangeShift */
    ]);
    let result = SfntHeader::from_reader_exact(&mut reader, 0, 12);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::IoError(_)));
    assert_eq!(err.to_string(), "failed to fill whole buffer");
}

#[test]
fn test_sfnt_header_read_exact_with_invalid_magic() {
    let mut reader = Cursor::new(vec![
        0x00, 0x00, 0x00, 0x00, // sfntVersion - INVALID magic
        0x00, 0x00, // numTables
        0x00, 0x00, // searchRange
        0x00, 0x00, // entrySelector
        0x00, 0x00, // rangeShift
    ]);
    let result = SfntHeader::from_reader_exact(&mut reader, 1, 12);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::UnknownMagic(0x00000000)));
}

#[test]
fn test_sfnt_header_write() {
    let header = SfntHeader {
        sfntVersion: Magic::TrueType,
        numTables: 0,
        searchRange: 0,
        entrySelector: 0,
        rangeShift: 0,
    };
    let mut buffer = Vec::new();
    header.write(&mut buffer).unwrap();
    assert_eq!(
        buffer,
        vec![
            0x00, 0x01, 0x00, 0x00, // sfntVersion
            0x00, 0x00, // numTables
            0x00, 0x00, // searchRange
            0x00, 0x00, // entrySelector
            0x00, 0x00, // rangeShift
        ]
    );
}

#[test]
fn test_sfnt_header_with_too_small_buffer() {
    let header = SfntHeader {
        sfntVersion: Magic::TrueType,
        numTables: 0,
        searchRange: 0,
        entrySelector: 0,
        rangeShift: 0,
    };
    let mut buffer = [0; 11];
    let mut cursor = Cursor::new(&mut buffer[..]);
    let result = header.write(&mut cursor);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::IoError(_)));
    assert_eq!(err.to_string(), "failed to write whole buffer");
}

#[test]
fn test_sfnt_header_checksum() {
    let header = SfntHeader {
        sfntVersion: Magic::TrueType,
        numTables: 3,
        searchRange: 4,
        entrySelector: 9,
        rangeShift: 9,
    };
    assert_eq!(
        header.checksum(),
        Wrapping(0x00010000u32 + 0x00030004u32 + 0x00090009u32)
    );
}

#[test]
fn test_sfnt_header_num_tables() {
    let header = SfntHeader {
        sfntVersion: Magic::TrueType,
        numTables: 3,
        searchRange: 4,
        entrySelector: 9,
        rangeShift: 9,
    };
    assert_eq!(header.num_tables(), 3);
}
