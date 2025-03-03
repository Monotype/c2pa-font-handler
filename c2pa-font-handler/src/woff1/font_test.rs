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

//! Tests for WOFF1 font.

use std::io::Cursor;

use super::Woff1Font;
use crate::{
    tag::FontTag, Font, FontDataRead, FontDirectory, FontTable,
    MutFontDataWrite,
};

#[test]
fn test_woff1_from_reader() {
    let woff_data = include_bytes!("../../../.devtools/font.woff");
    let mut woff_reader = Cursor::new(woff_data);
    let woff = Woff1Font::from_reader(&mut woff_reader).unwrap();
    assert_eq!(woff.tables.len(), 10);
    assert_eq!(woff.directory().entries().len(), 10);
    assert!(matches!(
        woff.header(),
        crate::woff1::header::Woff1Header {
            signature: 0x774f_4646,
            flavor: 0x4f54_544f,
            length: 0x0000_0000_0000_0374,
            numTables: 0x000a,
            reserved: 0x0000,
            totalSfntSize: 0x0000_0000_0000_0424,
            majorVersion: 0x0000,
            minorVersion: 0x0000,
            metaOffset: 0x0000_0000_0000_0000,
            metaLength: 0x0000_0000_0000_0000,
            metaOrigLength: 0x0000_0000_0000_0000,
            privOffset: 0x0000_0000_0000_0000,
            privLength: 0x0000_0000_0000_0000,
        }
    ));
    assert!(woff.contains_table(&FontTag::HEAD));
    assert_eq!(woff.table(&FontTag::HEAD).unwrap().data().len(), 52);
}

#[test]
fn test_woff1_write() {
    let woff_data = include_bytes!("../../../.devtools/font.woff");
    let mut woff_reader = Cursor::new(woff_data);
    let mut woff = Woff1Font::from_reader(&mut woff_reader).unwrap();
    let mut woff_writer = Cursor::new(Vec::new());
    woff.write(&mut woff_writer).unwrap();
    let woff_data = woff_writer.into_inner();
    let mut woff_reader = Cursor::new(woff_data);
    let woff = Woff1Font::from_reader(&mut woff_reader).unwrap();
    assert_eq!(woff.tables.len(), 10);
    assert_eq!(woff.directory().entries().len(), 10);
    assert!(matches!(
        woff.header(),
        crate::woff1::header::Woff1Header {
            signature: 0x774f_4646,
            flavor: 0x4f54_544f,
            length: 0x0000_0000_0000_0374,
            numTables: 0x000a,
            reserved: 0x0000,
            totalSfntSize: 0x0000_0000_0000_0424,
            majorVersion: 0x0000,
            minorVersion: 0x0000,
            metaOffset: 0x0000_0000_0000_0000,
            metaLength: 0x0000_0000_0000_0000,
            metaOrigLength: 0x0000_0000_0000_0000,
            privOffset: 0x0000_0000_0000_0000,
            privLength: 0x0000_0000_0000_0000,
        }
    ));
    assert!(woff.contains_table(&FontTag::HEAD));
    assert_eq!(woff.table(&FontTag::HEAD).unwrap().len(), 52);
}

#[test]
fn test_woff1_read_with_private_data() {
    // Simulate a WOFF font
    let woff_data = vec![
        0x77, 0x4f, 0x46, 0x46, // Signature
        0x4f, 0x54, 0x54, 0x4f, // Flavor
        0x00, 0x00, 0x00, 0x48, // Length
        0x00, 0x01, 0x00, 0x00, // Number of tables + Reserved
        0x00, 0x00, 0x00, 0x18, // Total sfnt size
        0x00, 0x00, 0x00, 0x00, // Major version + Minor version
        0x00, 0x00, 0x00, 0x00, // Metadata Offset
        0x00, 0x00, 0x00, 0x00, // Metadata Length
        0x00, 0x00, 0x00, 0x00, // Metadata Original Length
        0x00, 0x00, 0x00, 0x44, // Private Offset
        0x00, 0x00, 0x00, 0x04, // Private Length
        0x74, 0x65, 0x73, 0x74, // Directory entry - tag (test)
        0x00, 0x00, 0x00, 0x40, // Directory entry - offset
        0x00, 0x00, 0x00, 0x04, // Directory entry - comp length
        0x00, 0x00, 0x00, 0x04, // Directory entry - orig length
        0x00, 0x00, 0x00, 0x00, // Directory entry - orig checksum
        0x04, 0x03, 0x02, 0x01, // 'test' table
        0x77, 0x55, 0x33, 0x58, // Private data
    ];
    let mut woff_reader = Cursor::new(woff_data);
    let woff = Woff1Font::from_reader(&mut woff_reader).unwrap();
    assert_eq!(woff.tables.len(), 1);
    assert_eq!(woff.directory().entries().len(), 1);
    assert!(matches!(
        woff.header(),
        crate::woff1::header::Woff1Header {
            signature: 0x774f_4646,
            flavor: 0x4f54_544f,
            length: 0x0000_0048,
            numTables: 0x0001,
            reserved: 0x0000,
            totalSfntSize: 0x0000_0018,
            majorVersion: 0x0000,
            minorVersion: 0x0000,
            metaOffset: 0x0000_0000,
            metaLength: 0x0000_0000,
            metaOrigLength: 0x0000_0000,
            privOffset: 0x0000_0044,
            privLength: 0x0000_0004,
        }
    ));
    assert!(woff.contains_table(&FontTag::new(*b"test")));
    let table = woff.table(&FontTag::new(*b"test")).unwrap();
    assert_eq!(table.len(), 4);
    assert_eq!(table.data(), &[0x04, 0x03, 0x02, 0x01]);
    let private_data = woff.private_data.unwrap();
    assert_eq!(private_data.len(), 4);
    assert_eq!(private_data.data(), b"wU3X");
}

#[test]
fn test_woff1_write_with_private_data_non_4byte_aligned() {
    // Simulate a WOFF font
    let woff_data = vec![
        0x77, 0x4f, 0x46, 0x46, // Signature
        0x4f, 0x54, 0x54, 0x4f, // Flavor
        0x00, 0x00, 0x00, 0x49, // Length
        0x00, 0x01, 0x00, 0x00, // Number of tables + Reserved
        0x00, 0x00, 0x00, 0x18, // Total sfnt size
        0x00, 0x00, 0x00, 0x00, // Major version + Minor version
        0x00, 0x00, 0x00, 0x00, // Metadata Offset
        0x00, 0x00, 0x00, 0x00, // Metadata Length
        0x00, 0x00, 0x00, 0x00, // Metadata Original Length
        0x00, 0x00, 0x00, 0x44, // Private Offset
        0x00, 0x00, 0x00, 0x05, // Private Length
        0x74, 0x65, 0x73, 0x74, // Directory entry - tag (text)
        0x00, 0x00, 0x00, 0x40, // Directory entry - offset
        0x00, 0x00, 0x00, 0x04, // Directory entry - comp length
        0x00, 0x00, 0x00, 0x04, // Directory entry - orig length
        0x00, 0x00, 0x00, 0x00, // Directory entry - orig checksum
        0x04, 0x03, 0x02, 0x01, // 'test' table
        0x77, 0x55, 0x33, 0x58, // Private data
        0x00,
    ];
    // The simulated WOFF font is not 4-byte aligned, even though it should be
    assert_eq!(woff_data.len() % 4, 1);
    let mut woff_reader = Cursor::new(woff_data);
    // Create the WOFF font
    let mut woff = Woff1Font::from_reader(&mut woff_reader).unwrap();

    // Create a destination buffer for writing
    let mut destination = Cursor::new(Vec::new());
    let result = woff.write(&mut destination);
    assert!(result.is_ok());
    let woff_data = destination.into_inner();
    // Ensure the woff data is 4-byte aligned
    assert_eq!(woff_data.len() % 4, 0);
}

#[test]
fn test_woff1_read_with_metadata() {
    // Simulate a WOFF font
    let woff_data = vec![
        0x77, 0x4f, 0x46, 0x46, // Signature
        0x4f, 0x54, 0x54, 0x4f, // Flavor
        0x00, 0x00, 0x00, 0x48, // Length
        0x00, 0x01, 0x00, 0x00, // Number of tables + Reserved
        0x00, 0x00, 0x00, 0x18, // Total sfnt size
        0x00, 0x00, 0x00, 0x00, // Major version + Minor version
        0x00, 0x00, 0x00, 0x44, // Metadata Offset
        0x00, 0x00, 0x00, 0x04, // Metadata Length
        0x00, 0x00, 0x00, 0x04, // Metadata Original Length
        0x00, 0x00, 0x00, 0x00, // Private Offset
        0x00, 0x00, 0x00, 0x00, // Private Length
        0x74, 0x65, 0x73, 0x74, // Directory entry - tag (test)
        0x00, 0x00, 0x00, 0x40, // Directory entry - offset
        0x00, 0x00, 0x00, 0x04, // Directory entry - comp length
        0x00, 0x00, 0x00, 0x04, // Directory entry - orig length
        0x00, 0x00, 0x00, 0x00, // Directory entry - orig checksum
        0x04, 0x03, 0x02, 0x01, // 'test' table
        0x77, 0x55, 0x33, 0x58, // Metadata
    ];
    let mut woff_reader = Cursor::new(woff_data);
    let woff = Woff1Font::from_reader(&mut woff_reader).unwrap();
    assert_eq!(woff.tables.len(), 1);
    assert_eq!(woff.directory().entries().len(), 1);
    assert!(matches!(
        woff.header(),
        crate::woff1::header::Woff1Header {
            signature: 0x774f_4646,
            flavor: 0x4f54_544f,
            length: 0x0000_0048,
            numTables: 0x0001,
            reserved: 0x0000,
            totalSfntSize: 0x0000_0018,
            majorVersion: 0x0000,
            minorVersion: 0x0000,
            metaOffset: 0x0000_0044,
            metaLength: 0x0000_0004,
            metaOrigLength: 0x0000_0004,
            privOffset: 0x0000_0000,
            privLength: 0x0000_0000,
        }
    ));
    assert!(woff.contains_table(&FontTag::new(*b"test")));
    let table = woff.table(&FontTag::new(*b"test")).unwrap();
    assert_eq!(table.len(), 4);
    assert_eq!(table.data(), &[0x04, 0x03, 0x02, 0x01]);
    let metadata = woff.metadata.unwrap();
    assert_eq!(metadata.len(), 4);
    assert_eq!(metadata.data(), b"wU3X");
}

#[test]
fn test_woff1_write_with_metadata_non_4byte_aligned() {
    // Simulate a WOFF font
    let woff_data = vec![
        0x77, 0x4f, 0x46, 0x46, // Signature
        0x4f, 0x54, 0x54, 0x4f, // Flavor
        0x00, 0x00, 0x00, 0x50, // Length
        0x00, 0x01, 0x00, 0x00, // Number of tables + Reserved
        0x00, 0x00, 0x00, 0x18, // Total sfnt size
        0x00, 0x00, 0x00, 0x00, // Major version + Minor version
        0x00, 0x00, 0x00, 0x44, // Metadata Offset
        0x00, 0x00, 0x00, 0x05, // Metadata Length
        0x00, 0x00, 0x00, 0x05, // Metadata Original Length
        0x00, 0x00, 0x00, 0x4c, // Private Offset
        0x00, 0x00, 0x00, 0x04, // Private Length
        0x74, 0x65, 0x73, 0x74, // Directory entry - tag (text)
        0x00, 0x00, 0x00, 0x40, // Directory entry - offset
        0x00, 0x00, 0x00, 0x04, // Directory entry - comp length
        0x00, 0x00, 0x00, 0x04, // Directory entry - orig length
        0x00, 0x00, 0x00, 0x00, // Directory entry - orig checksum
        0x04, 0x03, 0x02, 0x01, // 'test' table
        0x77, 0x55, 0x33, 0x58, // Metadata
        0x00, 0x00, 0x00, 0x00, // Padding
        0x74, 0x65, 0x73, 0x74, // Private
    ];
    let mut woff_reader = Cursor::new(woff_data);
    // Create the WOFF font
    let mut woff = Woff1Font::from_reader(&mut woff_reader).unwrap();

    // Create a destination buffer for writing
    let mut destination = Cursor::new(Vec::new());
    let result = woff.write(&mut destination);
    assert!(result.is_ok());
    let woff_data = destination.into_inner();
    // Ensure the woff data is 4-byte aligned
    assert_eq!(woff_data.len() % 4, 0);
    // Read the WOFF font back
    let mut woff_reader = Cursor::new(woff_data);
    let woff = Woff1Font::from_reader(&mut woff_reader).unwrap();
    assert_eq!(woff.metadata.unwrap().data(), b"wU3X\x00");
    let private_data = woff.private_data.unwrap();
    assert_eq!(private_data.len(), 4);
    assert_eq!(private_data.data(), b"test");
}
