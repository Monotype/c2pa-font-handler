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
