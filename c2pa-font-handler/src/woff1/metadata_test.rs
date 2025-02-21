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

//! Tests for WOFF1 metadata extension module

use super::*;
use crate::{FontDataRead, MutFontDataWrite};

#[test]
fn test_woff_metadata_read_write() {
    let woff_data = include_bytes!("../../../.devtools/font.woff");
    let mut woff_reader = std::io::Cursor::new(woff_data);
    let mut woff = Woff1Font::from_reader(&mut woff_reader).unwrap();
    let mut woff_writer = std::io::Cursor::new(Vec::new());
    let metadata = Metadata::default();
    woff.set_metadata(metadata);
    woff.write(&mut woff_writer).unwrap();
    let mut woff_reader = std::io::Cursor::new(woff_writer.into_inner());
    let woff = Woff1Font::from_reader(&mut woff_reader).unwrap();
    let metadata = woff.metadata();
    assert!(metadata.is_some());
    let metadata = metadata.unwrap();
    println!("{:?}", metadata);
}
