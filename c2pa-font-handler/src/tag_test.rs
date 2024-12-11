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

//! Tests for table tags

use super::*;

#[test]
fn test_tag_read_exact() {
    let mut reader = std::io::Cursor::new(&b"bb2c");
    let result = FontTag::from_reader_exact(&mut reader, 0, 4);
    assert!(result.is_ok());
    let tag = result.unwrap();
    assert_eq!(tag.data(), *b"bb2c");
}

#[test]
fn test_tag_read_exact_with_bad_size() {
    let mut reader = std::io::Cursor::new(&b"bb2c");
    let result = FontTag::from_reader_exact(&mut reader, 0, 3);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::InvalidSizeForTAG(3)));
}

#[test]
fn test_tag_read_exact_with_invalid_sized_buffer() {
    let mut reader = std::io::Cursor::new(&b"bb2");
    let result = FontTag::from_reader_exact(&mut reader, 0, 4);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::IoError(_)));
    assert_eq!(err.to_string(), "failed to fill whole buffer");
}

#[test]
fn test_tag_write() {
    let tag = FontTag::new(*b"bb2c");
    let mut writer = std::io::Cursor::new(Vec::new());
    let result = tag.write(&mut writer);
    assert!(result.is_ok());
    assert_eq!(writer.into_inner(), b"bb2c");
}

#[test]
fn test_tag_display() {
    let tag = FontTag::new(*b"bb2c");
    assert_eq!(format!("{}", tag), "bb2c");
}

#[test]
fn test_tag_debug() {
    let tag = FontTag::new(*b"bb2c");
    assert_eq!(format!("{:?}", tag), "FontTag(bb2c)");
}
