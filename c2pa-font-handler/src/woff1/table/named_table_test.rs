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

//! Tests for the generic SFNT table module

use std::{io::Cursor, num::Wrapping};

use super::*;

#[test]
fn test_named_table_c2pa_read_exact() {
    let mut data = vec![];
    data.extend_from_slice(&[0x00, 0x01]); // major_version
    data.extend_from_slice(&[0x00, 0x04]); // minor_version
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // active manifest uri offset
    data.extend_from_slice(&[0x00, 0x00]); // active manifest uri length
    data.extend_from_slice(&[0x00, 0x00]); // reserved
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // content_credential offset
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // content_credential
    let mut reader = Cursor::new(data);
    let result =
        NamedTable::from_reader_exact(&FontTag::C2PA, &mut reader, 0, 20);
    assert!(result.is_ok());
    let c2pa = result.unwrap();
    assert!(matches!(c2pa, NamedTable::C2PA(_)));
}

#[test]
fn test_named_table_c2pa_len() {
    let c2pa = NamedTable::C2PA(TableC2PA::default());
    assert_eq!(c2pa.len(), 20);
    assert!(!c2pa.is_empty());
}

#[test]
fn test_named_table_c2pa_checksum() {
    let c2pa = NamedTable::C2PA(TableC2PA::default());
    let checksum = c2pa.checksum();
    let expected_checksum = Wrapping(0x00000001);
    assert_eq!(checksum, expected_checksum);
}

#[test]
fn test_named_table_c2pa_write() {
    let c2pa = NamedTable::C2PA(TableC2PA::default());
    let mut writer = Cursor::new(Vec::new());
    c2pa.write(&mut writer).unwrap();
    assert_eq!(
        writer.into_inner(),
        vec![
            0x00, 0x00, // major_version
            0x00, 0x01, // minor_version
            0x00, 0x00, 0x00, 0x00, // active manifest uri offset
            0x00, 0x00, // active manifest uri length
            0x00, 0x00, // reserved
            0x00, 0x00, 0x00, 0x00, // content_credential offset
            0x00, 0x00, 0x00, 0x00, // content_credential
        ]
    );
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
    let generic = NamedTable::Generic(Data {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    });
    assert_eq!(generic.len(), 8);
    assert!(!generic.is_empty());
}

#[test]
fn test_name_table_generic_checksum() {
    let generic = NamedTable::Generic(Data {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    });
    let checksum = generic.checksum();
    assert_eq!(checksum.0, 0x00000001);
}

#[test]
fn test_named_table_generic_write() {
    let generic = NamedTable::Generic(Data {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    });
    let mut buffer = Vec::new();
    let mut writer = Cursor::new(&mut buffer);
    generic.write(&mut writer).unwrap();
    let expected = vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00];
    assert_eq!(buffer, expected);
}

#[test]
fn test_display_of_named_tables() {
    let c2pa = NamedTable::C2PA(TableC2PA::default());
    let generic = NamedTable::Generic(Data {
        data: vec![0x00, 0x00, 0x00, 0x01],
    });
    assert_eq!(format!("{c2pa}"), "C2PA");
    assert_eq!(format!("{generic}"), "Generic(DATA)");
}
