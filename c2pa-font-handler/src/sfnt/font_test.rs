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

//! Tests for SFNT font.

use super::*;
use crate::{
    c2pa::{ContentCredentialRecord, UpdateContentCredentialRecord},
    error::FontIoError,
    sfnt::table::generic::TableGeneric,
};

#[test]
fn test_load_of_font() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let font = SfntFont::from_reader(&mut reader).unwrap();
    //assert_eq!(font.header.version(), 0x00010000);
    assert_eq!(font.header.num_tables(), 11);
    assert_eq!(font.tables.len(), 11);
}

#[test]
fn test_load_of_font_exact() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let font =
        SfntFont::from_reader_exact(&mut reader, 0, font_data.len()).unwrap();
    assert_eq!(font.header.num_tables(), 11);
    assert_eq!(font.tables.len(), 11);
}

#[test]
fn test_write_font_data_with_zero_tables() {
    let mut font = SfntFont {
        header: SfntHeader::default(),
        directory: SfntDirectory::new(),
        tables: std::collections::BTreeMap::new(),
    };
    let mut writer = std::io::Cursor::new(Vec::new());
    let result = font.write(&mut writer);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(
        err,
        FontIoError::SaveError(FontSaveError::NoTablesFound)
    ));
}

#[test]
fn test_load_font_with_bad_magic() {
    // mimic a bad font in memory
    let mut bad_font_data = [0u8; 100];
    // Add a bad magic number
    bad_font_data[0] = 0xff;
    bad_font_data[1] = 0xff;
    bad_font_data[2] = 0xff;
    bad_font_data[3] = 0xff;
    let mut reader = std::io::Cursor::new(&bad_font_data);
    let result = SfntHeader::from_reader(&mut reader);
    assert!(result.is_err());
    let err = result.err().unwrap();
    matches!(err, FontIoError::UnknownMagic(_));
}

#[test]
fn test_load_font_with_wrong_number_of_directory_entries() {
    // mimic a bad font in memory
    let mut bad_font_data = [0u8; 12];
    // make sure the magic number is correct (truetype)
    bad_font_data[0] = 0x00;
    bad_font_data[1] = 0x01;
    bad_font_data[2] = 0x00;
    bad_font_data[3] = 0x00;
    // Add a bad number of directory entries
    bad_font_data[4] = 0x00;
    bad_font_data[5] = 0x01;
    let mut reader = std::io::Cursor::new(&bad_font_data);
    let result = SfntFont::from_reader(&mut reader);
    assert!(result.is_err());
    let err = result.err().unwrap();
    println!("{:?}", err);
    assert!(matches!(err, FontIoError::IoError(_)));
}

#[test]
fn test_font_write() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let mut writer = std::io::Cursor::new(Vec::new());

    // Write the font to the writer
    font.write(&mut writer).unwrap();

    // Get the inner vector for comparison
    let written_data = writer.into_inner();
    // Verify the lengths are the same
    assert_eq!(font_data.len(), written_data.len());
    // Verify the data is the same, as we didn't change anything
    assert_eq!(font_data, written_data.as_slice());
}

#[test]
fn test_font_write_new_table_added() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let mut writer = std::io::Cursor::new(Vec::new());

    // Add a new table to the font
    let new_table = TableGeneric {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    };
    font.tables
        .insert(FontTag::new(*b"test"), NamedTable::Generic(new_table));

    let new_table = TableGeneric {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    };
    font.tables
        .insert(FontTag::new(*b"te5t"), NamedTable::Generic(new_table));

    // Write the font to the writer
    let result = font.write(&mut writer);

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(
        err,
        FontIoError::SaveError(FontSaveError::TooManyTablesAdded)
    ));
}

#[test]
fn test_write_font_without_c2pa() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let mut writer = std::io::Cursor::new(Vec::new());
    let result = font.write(&mut writer);
    assert!(result.is_ok());
    // Verify what is in writer
    let written_data = writer.into_inner();
    assert_eq!(font_data.len(), written_data.len());
    assert_eq!(font_data, written_data.as_slice());
}

#[test]
fn test_write_font_with_c2pa() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let record = ContentCredentialRecord::builder()
        .with_version(1, 4)
        .with_active_manifest_uri("https://example.com".to_string())
        .with_content_credential(vec![0x00, 0x01, 0x02, 0x03])
        .build()
        .unwrap();
    assert!(!font.has_c2pa());
    font.add_c2pa_record(record).unwrap();
    let mut writer = std::io::Cursor::new(Vec::new());
    let result = font.write(&mut writer);
    assert!(result.is_ok());
    // Verify what is in writer
    let written_data = writer.into_inner();
    let mut new_reader = std::io::Cursor::new(&written_data);
    let new_font = SfntFont::from_reader(&mut new_reader).unwrap();
    assert!(new_font.has_c2pa());
    assert_eq!(new_font.tables.len(), font.tables.len());
}

#[test]
fn test_font_write_table_deleted() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let mut writer = std::io::Cursor::new(Vec::new());

    // Remove a table from the font
    font.tables.remove(&FontTag::DSIG);
    font.tables.remove(&FontTag::HEAD);

    // Write the font to the writer
    let result = font.write(&mut writer);

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(
        err,
        FontIoError::SaveError(FontSaveError::TooManyTablesRemoved)
    ));
}

#[test]
fn test_font_stub_dsig() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();

    // Stub the DSIG table
    let result = font.stub_dsig();
    assert!(result.is_ok());

    // Verify the DSIG table is now a stub
    let dsig = font.tables.get(&FontTag::DSIG).unwrap();
    assert!(matches!(dsig, NamedTable::DSIG(_)));
}

#[test]
fn test_font_as_font_trait() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let font = SfntFont::from_reader(&mut reader).unwrap();
    assert_eq!(font.header().sfntVersion as u32, 0x4f54544f);
    assert_eq!(11, font.header().num_tables());
    assert_eq!(11, font.directory().entries().len());
    assert!(font.contains_table(&FontTag::new(*b"DSIG")));
    let table = font.table(&FontTag::new(*b"DSIG"));
    assert!(table.is_some());
    let table = table.unwrap();
    assert_eq!(table.len(), 8);
}

#[test]
fn test_adding_c2pa_record() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let record = ContentCredentialRecord::builder()
        .with_version(1, 4)
        .with_active_manifest_uri("https://example.com".to_string())
        .with_content_credential(vec![0x00, 0x01, 0x02, 0x03])
        .build()
        .unwrap();
    let result = font.add_c2pa_record(record);
    assert!(result.is_ok());
    assert!(font.has_c2pa());
    let result = font.get_c2pa();
    assert!(result.is_ok());
    let record = result.unwrap().unwrap();
    assert_eq!(record.major_version(), 1);
    assert_eq!(record.minor_version(), 4);
    assert_eq!(record.active_manifest_uri().unwrap(), "https://example.com");
    assert_eq!(
        record.content_credential().unwrap(),
        &[0x00, 0x01, 0x02, 0x03]
    );
}

#[test]
fn test_adding_c2pa_record_when_one_exists() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let record = ContentCredentialRecord::builder()
        .with_version(1, 4)
        .with_active_manifest_uri("https://example.com".to_string())
        .with_content_credential(vec![0x00, 0x01, 0x02, 0x03])
        .build()
        .unwrap();
    let result = font.add_c2pa_record(record.clone());
    assert!(result.is_ok());
    assert!(font.has_c2pa());
    let result = font.add_c2pa_record(record);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::ContentCredentialAlreadyExists));
}

#[test]
fn test_removing_c2pa_record() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let record = ContentCredentialRecord::builder()
        .with_version(1, 4)
        .with_active_manifest_uri("https://example.com".to_string())
        .with_content_credential(vec![0x00, 0x01, 0x02, 0x03])
        .build()
        .unwrap();
    let result = font.add_c2pa_record(record);
    assert!(result.is_ok());
    assert!(font.has_c2pa());
    let result = font.remove_c2pa_record();
    assert!(result.is_ok());
    let result = font.get_c2pa();
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.is_none());
}

#[test]
fn test_removing_c2pa_record_when_one_does_not_exists() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let result = font.remove_c2pa_record();
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::ContentCredentialNotFound));
}

#[test]
fn test_updating_c2pa_record_when_occupied() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let record = ContentCredentialRecord::builder()
        .with_version(1, 4)
        .with_active_manifest_uri("https://example.com".to_string())
        .with_content_credential(vec![0x00, 0x01, 0x02, 0x03])
        .build()
        .unwrap();
    font.add_c2pa_record(record).unwrap();
    let update_record = UpdateContentCredentialRecord::builder()
        .without_active_manifest_uri()
        .build();
    let result = font.update_c2pa_record(update_record);
    assert!(result.is_ok());
    assert!(font.has_c2pa());
    let result = font.get_c2pa();
    assert!(result.is_ok());
    let record = result.unwrap().unwrap();
    assert_eq!(record.major_version(), 1);
    assert_eq!(record.minor_version(), 4);
    assert_eq!(record.active_manifest_uri(), None);
    assert_eq!(
        record.content_credential().unwrap(),
        &[0x00, 0x01, 0x02, 0x03]
    );
}
#[test]
fn test_updating_c2pa_record_when_vacant() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = std::io::Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let update_record = UpdateContentCredentialRecord::builder()
        .without_active_manifest_uri()
        .build();
    let result = font.update_c2pa_record(update_record);
    assert!(result.is_ok());
    assert!(font.has_c2pa());
    let result = font.get_c2pa();
    assert!(result.is_ok());
    let record = result.unwrap().unwrap();
    assert_eq!(record.major_version(), 1);
    assert_eq!(record.minor_version(), 4);
    assert_eq!(record.active_manifest_uri(), None);
    assert_eq!(record.content_credential(), None);
}
