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

use std::{io::Cursor, num::Wrapping};

use super::*;

#[test]
fn test_table_generic_read_exact() {
    let mut reader = Cursor::new(vec![
        0x00, 0x00, 0x00, 0x01, // version
        0x00, 0x00, // numSignatures
        0x00, 0x00, // flags
    ]);
    let result = TableGeneric::from_reader_exact(&mut reader, 0, 8);
    assert!(result.is_ok());
    let generic = result.unwrap();
    assert_eq!(
        generic.data,
        vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]
    );
}

#[test]
fn test_reader_exact_with_invalid_sized_buffer() {
    let mut reader = Cursor::new(vec![0; 7]);
    let result = TableGeneric::from_reader_exact(&mut reader, 0, 8);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::IoError(_)));
    assert_eq!(err.to_string(), "failed to fill whole buffer");
}

#[test]
fn test_table_generic_len() {
    let generic = TableGeneric {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    };
    assert_eq!(generic.len(), 8);
}

#[test]
fn test_table_generic_checksum() {
    let generic = TableGeneric {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    };
    assert_eq!(generic.checksum(), Wrapping(0x00000001));
}

#[test]
fn test_table_generic_write() {
    let generic = TableGeneric {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    };
    let mut buffer = Vec::new();
    generic.write(&mut buffer).unwrap();
    assert_eq!(buffer, vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]);
}

#[test]
fn test_table_generic_write_with_4_byte_alignment() {
    // Create table with 5 bytes, which should be padded to 8 bytes
    let generic = TableGeneric {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00],
    };
    let mut buffer = Vec::new();
    generic.write(&mut buffer).unwrap();
    assert_eq!(buffer, vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]);
}
#[test]
fn test_table_generic_write_with_bad_buffer_size() {
    let generic = TableGeneric {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    };
    let mut buffer = [0; 7];
    let mut cursor = Cursor::new(&mut buffer[..]);
    let result = generic.write(&mut cursor);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::FailedToWriteTableData(_)));
}

#[test]
fn test_table_generic_write_with_bad_buffer_size_and_padding() {
    let generic = TableGeneric {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00],
    };
    let mut buffer = [0; 4];
    let mut cursor = Cursor::new(&mut buffer[..]);
    let result = generic.write(&mut cursor);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::FailedToWriteTableData(_)));
}
