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

//! Tests for the generic SFNT table module

use std::io::Cursor;

use super::*;

#[test]
fn test_named_table_dsig_read_exact() {
    let mut reader = Cursor::new(vec![
        0x00, 0x00, 0x00, 0x01, // version
        0x00, 0x00, // numSignatures
        0x00, 0x00, // flags
    ]);
    let result =
        NamedTable::from_reader_exact(&FontTag::DSIG, &mut reader, 0, 8);
    assert!(result.is_ok());
    let dsig = result.unwrap();
    assert!(matches!(dsig, NamedTable::DSIG(_)));
}

#[test]
fn test_named_table_dsig_len() {
    let dsig = NamedTable::DSIG(TableDSIG {
        version: 1,
        numSignatures: 0,
        flags: 0,
        data: vec![],
    });
    assert_eq!(dsig.len(), 8);
}

#[test]
fn test_named_table_dsig_checksum() {
    let dsig = NamedTable::DSIG(TableDSIG {
        version: 1,
        numSignatures: 0,
        flags: 0,
        data: vec![],
    });
    let checksum = dsig.checksum();
    assert_eq!(
        checksum.0,
        0x00000001 /* Version */
                   /* Commented out because of clippy
                       + 0x00000000 // Num signatures
                       + 0x00000000 // Flags
                   */
    );
}

#[test]
fn test_named_table_dsig_write() {
    let dsig = NamedTable::DSIG(TableDSIG {
        version: 1,
        numSignatures: 0,
        flags: 0,
        data: vec![],
    });
    let mut writer = Cursor::new(Vec::new());
    dsig.write(&mut writer).unwrap();
    assert_eq!(
        writer.into_inner(),
        vec![
            0x00, 0x00, 0x00, 0x01, // version
            0x00, 0x00, // numSignatures
            0x00, 0x00, // flags
        ]
    );
}

#[test]
fn test_named_table_head_read_exact() {
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
    let result =
        NamedTable::from_reader_exact(&FontTag::HEAD, &mut reader, 0, 54);
    println!("{:?}", &result.as_ref().err());
    assert!(result.is_ok());
    let head = result.unwrap();
    assert!(matches!(head, NamedTable::Head(_)));
}

#[test]
fn test_named_table_head_len() {
    let head = NamedTable::Head(TableHead {
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
    });
    assert_eq!(head.len(), 54);
}

#[test]
fn test_named_table_head_checksum() {
    let head = NamedTable::Head(TableHead {
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
    });
    let checksum = head.checksum();
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
fn test_named_table_head_write() {
    let head = NamedTable::Head(TableHead {
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
    });
    let mut buffer = Vec::new();
    let mut writer = Cursor::new(&mut buffer);
    head.write(&mut writer).unwrap();
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
fn test_named_table_generic_read_exact() {
    let mut reader = Cursor::new(vec![
        0x00, 0x00, 0x00, 0x01, // version
        0x00, 0x00, // numSignatures
        0x00, 0x00, // flags
    ]);
    let result = NamedTable::from_reader_exact(
        &FontTag::new(*b"    "),
        &mut reader,
        0,
        8,
    );
    assert!(result.is_ok());
    let generic = result.unwrap();
    assert!(matches!(generic, NamedTable::Generic(_)));
}

#[test]
fn test_named_table_generic_len() {
    let generic = NamedTable::Generic(TableGeneric {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    });
    assert_eq!(generic.len(), 8);
}

#[test]
fn test_name_table_generic_checksum() {
    let generic = NamedTable::Generic(TableGeneric {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    });
    let checksum = generic.checksum();
    assert_eq!(checksum.0, 0x00000001);
}

#[test]
fn test_named_table_generic_write() {
    let generic = NamedTable::Generic(TableGeneric {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    });
    let mut buffer = Vec::new();
    let mut writer = Cursor::new(&mut buffer);
    generic.write(&mut writer).unwrap();
    let expected = vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00];
    assert_eq!(buffer, expected);
}
