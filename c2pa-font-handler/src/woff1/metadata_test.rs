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
    let metadata = Metadata {
        version: "1.0".to_string(),
        unique_ids: Some(vec![UniqueId {
            id: "unique_id".to_string(),
        }]),
        vendor: Some(vec![Vendor {
            name: "vendor".to_string(),
            url: Some("https://example.com/vendor".to_string()),
            ..Default::default()
        }]),
        credits: Some(Credits {
            credits: vec![
                Credit {
                    name: "credit".to_string(),
                    url: Some("https://example.com/credit".to_string()),
                    ..Default::default()
                },
                Credit {
                    name: "credit2".to_string(),
                    url: Some("https://example.com/credit2".to_string()),
                    ..Default::default()
                },
            ],
        }),
        description: Some(vec![Description {
            url: Some("https://example.com/description".to_string()),
            text: vec![Text {
                text: "A member of the demo font family".to_string(),
                xml_lang: Some("en".to_string()),
                dir: Some(TextDirection::RightToLeft),
                class: Some(vec!["test_class".to_string()]),
            }],
        }]),
        ..Default::default()
    };
    println!("{:?}", metadata);
    woff.set_metadata(metadata).unwrap();
    woff.write(&mut woff_writer).unwrap();
    let mut woff_reader = std::io::Cursor::new(woff_writer.into_inner());
    let woff = Woff1Font::from_reader(&mut woff_reader).unwrap();
    let result = woff.metadata();
    println!("{:?}", result);
    assert!(result.is_ok());
    let metadata = result.unwrap();
    assert!(metadata.is_some());
    let metadata = metadata.unwrap();
    println!("{:?}", metadata);
}
