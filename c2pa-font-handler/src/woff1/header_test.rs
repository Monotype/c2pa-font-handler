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

//! Tests for WOFF1 header module

use super::*;

#[test]
fn test_woff1_header_default() {
    let woff = Woff1Header::default();
    assert!(matches!(
        woff,
        Woff1Header {
            signature: Magic::Woff,
            flavor: 0,
            length: 0,
            numTables: 0,
            reserved: 0,
            totalSfntSize: 0,
            majorVersion: 0,
            minorVersion: 0,
            metaOffset: 0,
            metaLength: 0,
            metaOrigLength: 0,
            privOffset: 0,
            privLength: 0,
        }
    ));
}

#[test]
fn test_woff1_header_read_exact() {
    let woff_data = include_bytes!("../../../.devtools/font.woff");
    let mut woff_reader = std::io::Cursor::new(woff_data);
    let woff =
        Woff1Header::from_reader_exact(&mut woff_reader, 0, Woff1Header::SIZE)
            .unwrap();
    assert!(matches!(
        woff,
        Woff1Header {
            signature: Magic::Woff,
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
}

#[test]
fn test_woff1_header_read_exact_with_bad_size() {
    let woff_data = include_bytes!("../../../.devtools/font.woff");
    let mut woff_reader = std::io::Cursor::new(woff_data);
    let result = Woff1Header::from_reader_exact(&mut woff_reader, 0, 1);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        FontIoError::InvalidSizeForHeader(1)
    ));
}

#[test]
fn test_woff1_header_read_exact_too_small_buffer() {
    let woff_data = include_bytes!("../../../.devtools/font.woff");
    // Get a slice of the first 10 bytes
    let woff_slice = &woff_data[0..10];
    let mut woff_reader = std::io::Cursor::new(woff_slice);
    let result =
        Woff1Header::from_reader_exact(&mut woff_reader, 0, Woff1Header::SIZE);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), FontIoError::IoError(_)));
}

#[test]
fn test_woff1_header_write() {
    let woff = Woff1Header {
        signature: Magic::Woff,
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
    };
    // Create a writer
    let mut woff_writer = std::io::Cursor::new(Vec::new());
    // Write the WOFF1 header
    woff.write(&mut woff_writer).unwrap();
    // Get the written data
    let woff_data = woff_writer.into_inner();
    // Create a reader around the data
    let mut woff_reader = std::io::Cursor::new(woff_data);
    // And read in the header
    let woff =
        Woff1Header::from_reader_exact(&mut woff_reader, 0, Woff1Header::SIZE)
            .unwrap();
    assert!(matches!(
        woff,
        Woff1Header {
            signature: Magic::Woff,
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
}

#[test]
fn test_woff1_header_checksum() {
    let woff = Woff1Header {
        signature: Magic::Woff,
        flavor: 0x4f54_544f,
        length: 0x0000_0374,
        numTables: 0x0000_000a,
        reserved: 0x0000_0000,
        totalSfntSize: 0x0000_0000_0000_0424,
        majorVersion: 0x0000_0000,
        minorVersion: 0x0000_0000,
        metaOffset: 0x0000_0000_0000_0000,
        metaLength: 0x0000_0000_0000_0000,
        metaOrigLength: 0x0000_0000_0000_0000,
        privOffset: 0x0000_0000_0000_0000,
        privLength: 0x0000_0000_0000_0000,
    };
    let checksum = woff.checksum();
    let expected = Wrapping(0x774f_4646_u32) // signature
    + Wrapping(0x4f54_544f) // flavor
    + Wrapping(0x0000_0374) // length
    + Wrapping(0x000a_0000) // numTables + reserved
    + Wrapping(0x0000_0424) // totalSfntSize
    + Wrapping(0x0000_0000) // majorVersion + minorVersion
    + Wrapping(0x0000_0000) // metaOffset
    + Wrapping(0x0000_0000) // metaLength
    + Wrapping(0x0000_0000) // metaOrigLength
    + Wrapping(0x0000_0000) // privOffset
    + Wrapping(0x0000_0000); // privLength
    assert_eq!(checksum, expected);
}

#[test]
fn test_woff1_header_num_tables() {
    let woff = Woff1Header {
        signature: Magic::Woff,
        flavor: 0,
        length: 0,
        numTables: 0x000a,
        reserved: 0,
        totalSfntSize: 0,
        majorVersion: 0,
        minorVersion: 0,
        metaOffset: 0,
        metaLength: 0,
        metaOrigLength: 0,
        privOffset: 0,
        privLength: 0,
    };
    assert_eq!(woff.num_tables(), 0x000a);
}
