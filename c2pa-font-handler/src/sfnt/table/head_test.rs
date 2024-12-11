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

//! Tests for the 'head' SFNT table module
use std::io::Cursor;

use super::*;

#[test]
fn test_reader_exact_with_bad_size() {
    let mut reader = std::io::Cursor::new(vec![0; 10]);
    let result = TableHead::from_reader_exact(&mut reader, 0, 11);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(
        err,
        FontIoError::LoadTableTruncated(FontTag::HEAD)
    ));
}

#[test]
fn test_reader_exact_with_invalid_sized_buffer() {
    // Get the size of a table head unit
    let size = TableHead::SIZE;
    // Create a buffer that is one byte short of the required size.
    let mut reader = std::io::Cursor::new(vec![0; size - 1]);
    // Read the table header from the buffer, which is not quite large enough
    // and should cause an I/O error.
    let result = TableHead::from_reader_exact(&mut reader, 0, size);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::IoError(_)));
    assert_eq!(err.to_string(), "failed to fill whole buffer");
}

#[test]
fn test_reader_with_bad_magic_number() {
    let mut reader = std::io::Cursor::new(vec![0; TableHead::SIZE]);
    let result = TableHead::from_reader(&mut reader);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::InvalidHeadMagicNumber(0u32)));
}

#[test]
fn test_reader_with_valid_data() {
    let mut reader = std::io::Cursor::new(vec![
        0x00, 0x01, // major version
        0x00, 0x01, // minor version
        0x01, 0x20, 0x30, 0x40, // font revision
        0x12, 0x98, 0x34, 0x76, // checksum adjustment
        0x5f, 0x0f, 0x3c, 0xf5, // magic number
        0xda, 0xda, // flags
        0x00, 0xf0, // units per em
        0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // created
        0x00, 0x00, 0x01, 0x20, 0x00, 0x00, 0x00, 0x00, // modified
        0x00, 0x00, // x min
        0x00, 0x09, // y min
        0x09, 0x00, // x max
        0x0a, 0x00, // y max
        0x12, 0x34, // mac style
        0x09, 0xf2, // lowest rec ppem
        0x07, 0x07, // font direction hint
        0x0b, 0x20, // index to loc format
        0x02, 0x3d, // glyph data format
        0x00, 0x00, // padding
    ]);
    let result = TableHead::from_reader(&mut reader);
    assert!(result.is_ok());
    let result = result.unwrap();
    let major_version = result.majorVersion;
    assert_eq!(major_version, 1u16);
    let minor_version = result.minorVersion;
    assert_eq!(minor_version, 1u16);
    let font_revision = result.fontRevision;
    assert_eq!(font_revision, 0x01203040u32);
    let checksum_adjustment = result.checksumAdjustment;
    assert_eq!(checksum_adjustment, 0x12983476u32);
    let magic_number = result.magicNumber;
    assert_eq!(magic_number, 0x5f0f3cf5u32);
    let flags = result.flags;
    assert_eq!(flags, 0xdadau16);
    let units_per_em = result.unitsPerEm;
    assert_eq!(units_per_em, 0x00f0u16);
    let created = result.created;
    assert_eq!(created, 0x0001000000000000i64);
    let modified = result.modified;
    assert_eq!(modified, 0x0000012000000000i64);
    let x_min = result.xMin;
    assert_eq!(x_min, 0i16);
    let y_min = result.yMin;
    assert_eq!(y_min, 0x0009i16);
    let x_max = result.xMax;
    assert_eq!(x_max, 0x0900i16);
    let y_max = result.yMax;
    assert_eq!(y_max, 0x0a00i16);
    let mac_style = result.macStyle;
    assert_eq!(mac_style, 0x1234u16);
    let lowest_rec_ppem = result.lowestRecPPEM;
    assert_eq!(lowest_rec_ppem, 0x09f2u16);
    let font_direction_hint = result.fontDirectionHint;
    assert_eq!(font_direction_hint, 0x0707i16);
    let index_to_loc_format = result.indexToLocFormat;
    assert_eq!(index_to_loc_format, 0x0b20i16);
    let glyph_data_format = result.glyphDataFormat;
    assert_eq!(glyph_data_format, 0x023di16);
}

#[test]
fn test_table_head_len() {
    let table = TableHead {
        majorVersion: 1,
        minorVersion: 0,
        fontRevision: 0x12345678,
        checksumAdjustment: 0x9abcdef0,
        magicNumber: 0x5f0f3cf5,
        flags: 0x1234,
        unitsPerEm: 0x00f0,
        created: 0x0001000000000000,
        modified: 0x0000012000000000,
        xMin: 0,
        yMin: 0x0009,
        xMax: 0x0900,
        yMax: 0x0a00,
        macStyle: 0x1234,
        lowestRecPPEM: 0x09f2,
        fontDirectionHint: 0x0707,
        indexToLocFormat: 0x0b20,
        glyphDataFormat: 0x023d,
    };
    let len = table.len();
    assert_eq!(len, 54u32);
}

#[test]
fn test_table_head_checksum() {
    let table = TableHead {
        majorVersion: 1,
        minorVersion: 0,
        fontRevision: 0x12345678,
        checksumAdjustment: 0x9abcdef0,
        magicNumber: 0x5f0f3cf5,
        flags: 0x1234,
        unitsPerEm: 0x00f0,
        created: 0x0001000000000000,
        modified: 0x0000012000000000,
        xMin: 0,
        yMin: 0x0009,
        xMax: 0x0900,
        yMax: 0x0a00,
        macStyle: 0x1234,
        lowestRecPPEM: 0x09f2,
        fontDirectionHint: 0x0707,
        indexToLocFormat: 0x0b20,
        glyphDataFormat: 0x023d,
    };
    let checksum = table.checksum();
    assert_eq!(
        checksum.0,
        0x00010000 // Major + Minor version
            + 0x12345678 // Font revision
            //+ 0x9abcdef0 // Checksum adjustment - skipped
            + 0x5f0f3cf5 // Magic number
            + 0x123400f0 // Flags + Units per em
            + 0x00010000 // Created (low)
            // + 0x00000000 // Created (high) /* commented out because of clippy */
            + 0x00000120 // Modified (low)
            // + 0x00000000 // Modified (high) /* commented out because of clippy */
            + 0x00000009 // xMin + yMin
            + 0x09000a00 // xMax + yMax
            + 0x123409f2 // macStyle + lowestRecPPEM
            + 0x07070b20 // fontDirectionHint + indexToLocFormat
            + 0x023d0000 // glyphDataFormat + padding
    );
}

#[test]
fn test_font_header_write() {
    let table = TableHead {
        majorVersion: 1,
        minorVersion: 0,
        fontRevision: 0x12345678,
        checksumAdjustment: 0x9abcdef0,
        magicNumber: 0x5f0f3cf5,
        flags: 0x1234,
        unitsPerEm: 0x00f0,
        created: 0x0001000000000000,
        modified: 0x0000012000000000,
        xMin: 0,
        yMin: 0x0009,
        xMax: 0x0900,
        yMax: 0x0a00,
        macStyle: 0x1234,
        lowestRecPPEM: 0x09f2,
        fontDirectionHint: 0x0707,
        indexToLocFormat: 0x0b20,
        glyphDataFormat: 0x023d,
    };
    let mut buffer = Vec::new();
    let result = table.write(&mut buffer);
    assert!(result.is_ok());
    assert_eq!(buffer.len(), 56);
    let expected = vec![
        0x00, 0x01, // major version
        0x00, 0x00, // minor version
        0x12, 0x34, 0x56, 0x78, // font revision
        0x9a, 0xbc, 0xde, 0xf0, // Checksum Adjustment
        0x5f, 0x0f, 0x3c, 0xf5, // magic number
        0x12, 0x34, // flags
        0x00, 0xf0, // units per em
        0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // created
        0x00, 0x00, 0x01, 0x20, 0x00, 0x00, 0x00, 0x00, // modified
        0x00, 0x00, // x min
        0x00, 0x09, // y min
        0x09, 0x00, // x max
        0x0a, 0x00, // y max
        0x12, 0x34, // mac style
        0x09, 0xf2, // lowest rec ppem
        0x07, 0x07, // font direction hint
        0x0b, 0x20, // index to loc format
        0x02, 0x3d, // glyph data format
        0x00, 0x00, // padding
    ];
    assert_eq!(buffer, expected);
}

#[test]
fn test_font_header_write_with_too_small_buffer() {
    let table = TableHead {
        majorVersion: 1,
        minorVersion: 0,
        fontRevision: 0x12345678,
        checksumAdjustment: 0x9abcdef0,
        magicNumber: 0x5f0f3cf5,
        flags: 0x1234,
        unitsPerEm: 0x00f0,
        created: 0x0001000000000000,
        modified: 0x0000012000000000,
        xMin: 0,
        yMin: 0x0009,
        xMax: 0x0900,
        yMax: 0x0a00,
        macStyle: 0x1234,
        lowestRecPPEM: 0x09f2,
        fontDirectionHint: 0x0707,
        indexToLocFormat: 0x0b20,
        glyphDataFormat: 0x023d,
    };
    let mut buffer = [0; 50];
    let mut cursor = Cursor::new(&mut buffer[..]);
    let result = table.write(&mut cursor);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::IoError(_)));
    assert_eq!(err.to_string(), "failed to write whole buffer");
}

#[test]
fn test_table_header_debug_fmt() {
    let table = TableHead {
        majorVersion: 1,
        minorVersion: 0,
        fontRevision: 0x12345678,
        checksumAdjustment: 0x9abcdef0,
        magicNumber: 0x5f0f3cf5,
        flags: 0x1234,
        unitsPerEm: 0x00f0,
        created: 0x0001000000000000,
        modified: 0x0000012000000000,
        xMin: 0,
        yMin: 0x0009,
        xMax: 0x0900,
        yMax: 0x0a00,
        macStyle: 0x1234,
        lowestRecPPEM: 0x09f2,
        fontDirectionHint: 0x0707,
        indexToLocFormat: 0x0b20,
        glyphDataFormat: 0x023d,
    };
    let debug_fmt = format!("{:?}", table);
    let expected = "TableHead { majorVersion: 1, minorVersion: 0, fontRevision: 305419896, checksumAdjustment: 2596069104, magicNumber: 1594834165, flags: 4660, unitsPerEm: 240, created: 281474976710656, modified: 1236950581248, xMin: 0, yMin: 9, xMax: 2304, yMax: 2560, macStyle: 4660, lowestRecPPEM: 2546, fontDirectionHint: 1799, indexToLocFormat: 2848, glyphDataFormat: 573 }";
    assert_eq!(debug_fmt, expected);
}
