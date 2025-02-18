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

//! Tests for the 'DSIG' SFNT table module

use std::io::Cursor;

use super::*;

#[test]
fn test_stub_dsig() {
    let stub = TableDSIG::stub();
    assert_eq!(stub.version, 1);
    assert_eq!(stub.numSignatures, 0);
    assert_eq!(stub.flags, 1);
}

#[test]
fn test_table_dsig_read() {
    let mut reader = Cursor::new(vec![
        0x00, 0x00, 0x00, 0x01, // version
        0x00, 0x00, // numSignatures
        0x00, 0x00, // flags
    ]);
    let result = TableDSIG::from_reader_exact(&mut reader, 0, 8);
    assert!(result.is_ok());
    let dsig = result.unwrap();
    assert_eq!(dsig.version, 1);
    assert_eq!(dsig.numSignatures, 0);
    assert_eq!(dsig.flags, 0);
}

#[test]
fn test_table_dsig_read_exact() {
    let mut reader = Cursor::new(vec![
        0x00, 0x00, 0x00, 0x01, // version
        0x00, 0x00, // numSignatures
        0x00, 0x00, // flags
    ]);
    let result = TableDSIG::from_reader_exact(&mut reader, 0, 8);
    assert!(result.is_ok());
    let dsig = result.unwrap();
    assert_eq!(dsig.version, 1);
    assert_eq!(dsig.numSignatures, 0);
    assert_eq!(dsig.flags, 0);
}

#[test]
fn test_table_dsig_read_exact_with_bad_size() {
    let mut reader = Cursor::new(vec![
        0x00, 0x00, 0x00, 0x01, // version
        0x00, 0x00, // numSignatures
        0x00, 0x00, // flags
    ]);
    let result = TableDSIG::from_reader_exact(&mut reader, 0, 7);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(
        err,
        FontIoError::LoadTableTruncated(FontTag::DSIG)
    ));
}

#[test]
fn test_table_dsig_read_exact_with_invalid_sized_buffer() {
    let mut reader = Cursor::new(vec![0; 7]);
    let result = TableDSIG::from_reader_exact(&mut reader, 0, 8);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::IoError(_)));
    assert_eq!(err.to_string(), "failed to fill whole buffer");
}

#[test]
fn test_table_dsig_write() {
    let dsig = TableDSIG {
        version: 1,
        numSignatures: 0,
        flags: 0,
        data: vec![],
    };
    let mut buffer = Vec::new();
    dsig.write(&mut buffer).unwrap();
    assert_eq!(
        buffer,
        vec![
            0x00, 0x00, 0x00, 0x01, // version
            0x00, 0x00, // numSignatures
            0x00, 0x00, // flags
        ]
    );
}

#[test]
fn test_table_dsig_write_with_too_small_buffer() {
    let dsig = TableDSIG {
        version: 1,
        numSignatures: 0,
        flags: 0,
        data: vec![],
    };
    let mut buffer = [0; 7];
    let mut cursor = Cursor::new(&mut buffer[..]);
    let result = dsig.write(&mut cursor);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::IoError(_)));
    assert_eq!(err.to_string(), "failed to write whole buffer");
}

#[test]
fn test_table_dsig_checksum() {
    let dsig = TableDSIG {
        version: 1,
        numSignatures: 0x12,
        flags: 0x87,
        data: vec![],
    };
    let checksum = dsig.checksum();
    assert_eq!(checksum, std::num::Wrapping(0x00000001 + 0x00120087));
}

#[test]
fn test_table_dsig_length() {
    let dsig = TableDSIG {
        version: 1,
        numSignatures: 0x12,
        flags: 0x87,
        data: vec![],
    };
    assert_eq!(dsig.len(), 8);
}
