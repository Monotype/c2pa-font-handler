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

use std::io::Cursor;

use super::*;
use crate::{
    c2pa::{ContentCredentialRecord, UpdateContentCredentialRecord},
    chunks::ChunkTypeTrait,
    data::Data,
    error::FontIoError,
};

#[test]
fn test_load_of_font() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = Cursor::new(font_data);
    let font = SfntFont::from_reader(&mut reader).unwrap();
    //assert_eq!(font.header.version(), 0x00010000);
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
    let mut writer = Cursor::new(Vec::new());
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
    let mut reader = Cursor::new(&bad_font_data);
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
    let mut reader = Cursor::new(&bad_font_data);
    let result = SfntFont::from_reader(&mut reader);
    assert!(result.is_err());
    let err = result.err().unwrap();
    println!("{err:?}");
    assert!(matches!(err, FontIoError::IoError(_)));
}

#[test]
fn test_font_write() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let mut writer = Cursor::new(Vec::new());

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
    let mut reader = Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let mut writer = Cursor::new(Vec::new());

    // Add a new table to the font
    let new_table = Data {
        data: vec![0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
    };
    font.tables
        .insert(FontTag::new(*b"test"), NamedTable::Generic(new_table));

    let new_table = Data {
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
    let mut reader = Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let mut writer = Cursor::new(Vec::new());
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
    let mut reader = Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let record = ContentCredentialRecord::builder()
        .with_version(0, 1)
        .with_active_manifest_uri("https://example.com".to_string())
        .with_content_credential(vec![0x00, 0x01, 0x02, 0x03])
        .build()
        .unwrap();
    assert!(!font.has_c2pa());
    font.add_c2pa_record(record).unwrap();
    let mut writer = Cursor::new(Vec::new());
    let result = font.write(&mut writer);
    assert!(result.is_ok());
    // Verify what is in writer
    let written_data = writer.into_inner();
    let mut new_reader = Cursor::new(&written_data);
    let new_font = SfntFont::from_reader(&mut new_reader).unwrap();
    assert!(new_font.has_c2pa());
    assert_eq!(new_font.tables.len(), font.tables.len());
}

#[test]
fn test_font_write_table_deleted() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let mut writer = Cursor::new(Vec::new());

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
    let mut reader = Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();

    // Stub the DSIG table
    let result = font.stub_dsig();
    assert!(result.is_ok());

    // Verify the DSIG table is now a stub
    let dsig = font.tables.get(&FontTag::DSIG).unwrap();
    assert!(matches!(dsig, NamedTable::DSIG(_)));
}

#[test]
#[tracing_test::traced_test]
fn test_font_stub_dsig_stream_not_present() {
    let font_data = vec![
        0x00, 0x01, 0x00, 0x00, // sfntVersion
        0x00, 0x01, // numTables
        0x00, 0x00, // searchRange
        0x00, 0x00, // entrySelector
        0x00, 0x00, // rangeShift
        // One table directory entry for a non-DSIG table
        0x74, 0x65, 0x73, 0x74, // tag 'test'
        0x00, 0x00, 0x00, 0x40, // offset
        0x00, 0x00, 0x00, 0x04, // comp length
        0x00, 0x00, 0x00, 0x04, // orig length
    ];
    let mut reader = Cursor::new(font_data);
    let mut destination = Cursor::new(Vec::new());

    // Attempt to stub the DSIG table when it is not present
    let result = stub_dsig_stream(&mut reader, &mut destination);
    assert!(result.is_ok());
    assert!(logs_contain("DSIG table is not present."));
    assert!(logs_contain(
        "DSIG table is not present or already stubbed, copying stream."
    ));

    // Rewind the destination to read it back
    destination.set_position(0);
    let font = SfntFont::from_reader(&mut destination).unwrap();
    let dsig = font.tables.get(&FontTag::DSIG);
    assert!(dsig.is_none());
}

#[test]
#[tracing_test::traced_test]
fn test_font_stub_dsig_stream_stubbed() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = Cursor::new(font_data);
    let mut destination = Cursor::new(Vec::new());

    // Stub the DSIG table
    let result = stub_dsig_stream(&mut reader, &mut destination);
    assert!(result.is_ok());

    // Rewind the destination to read it back
    destination.set_position(0);
    let font = SfntFont::from_reader(&mut destination).unwrap();
    // Verify the DSIG table is now a stub
    let dsig = font.tables.get(&FontTag::DSIG).unwrap();
    assert!(matches!(dsig, NamedTable::DSIG(_)));
    assert!(logs_contain("DSIG table is stubbed."));
    assert!(logs_contain(
        "DSIG table is not present or already stubbed, copying stream."
    ));
}

#[test]
#[tracing_test::traced_test]
fn test_font_stub_dsig_stream_present() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let dsig_table = NamedTable::DSIG(TableDSIG {
        version: 1,
        numSignatures: 1,
        flags: TableDSIG::DO_NOT_RESIGN,
        data: vec![0x01, 0x02, 0x03, 0x04],
    });
    font.tables.insert(FontTag::DSIG, dsig_table);
    let mut reader = Cursor::new(Vec::new());
    font.write(&mut reader).unwrap();
    reader.set_position(0);
    let mut destination = Cursor::new(Vec::new());

    // Stub the DSIG table
    let result = stub_dsig_stream(&mut reader, &mut destination);
    assert!(result.is_ok());

    destination.set_position(0);
    let font = SfntFont::from_reader(&mut destination).unwrap();
    // Verify the DSIG table is now a stub
    let dsig = font.tables.get(&FontTag::DSIG).unwrap();
    assert!(matches!(dsig, NamedTable::DSIG(_)));
    assert!(logs_contain("DSIG table is present and not stubbed."));
    assert!(logs_contain(
        "DSIG table is present and not stubbed, proceeding to stub it."
    ));
}

#[test]
fn test_font_as_font_trait() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = Cursor::new(font_data);
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
    let mut reader = Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let record = ContentCredentialRecord::builder()
        .with_version(0, 1)
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
    assert_eq!(record.major_version(), 0);
    assert_eq!(record.minor_version(), 1);
    assert_eq!(record.active_manifest_uri().unwrap(), "https://example.com");
    assert_eq!(
        record.content_credential().unwrap(),
        &[0x00, 0x01, 0x02, 0x03]
    );
}

#[test]
fn test_adding_c2pa_record_when_one_exists() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let record = ContentCredentialRecord::builder()
        .with_version(0, 1)
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
    let mut reader = Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let record = ContentCredentialRecord::builder()
        .with_version(0, 1)
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
    let mut reader = Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let result = font.remove_c2pa_record();
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, FontIoError::ContentCredentialNotFound));
}

#[test]
fn test_updating_c2pa_record_when_occupied() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = Cursor::new(font_data);
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    let record = ContentCredentialRecord::builder()
        .with_version(0, 1)
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
    assert_eq!(record.major_version(), 0);
    assert_eq!(record.minor_version(), 1);
    assert_eq!(record.active_manifest_uri(), None);
    assert_eq!(
        record.content_credential().unwrap(),
        &[0x00, 0x01, 0x02, 0x03]
    );
}
#[test]
fn test_updating_c2pa_record_when_vacant() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = Cursor::new(font_data);
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
    assert_eq!(record.major_version(), 0);
    assert_eq!(record.minor_version(), 1);
    assert_eq!(record.active_manifest_uri(), None);
    assert_eq!(record.content_credential(), None);
}

#[test]
fn test_sfnt_font_chunk_reader_bad_header() {
    let mut reader = Cursor::new(vec![0u8; 10]);
    let result = SfntFont::get_chunk_positions(&mut reader);
    assert!(result.is_err());
    let err = result.err().unwrap();
    // Since we didn't do an appropriate magic for the font, we should get an
    // unknown magic error.
    assert!(matches!(err, FontIoError::UnknownMagic(_)));
}

#[test]
fn test_sfnt_font_chunk_reader_bad_directory() {
    // Mimic a bad font in memory, where the directory is too short
    let mut reader = Cursor::new(vec![
        // Simulate the magic number
        0x00, 0x01, 0x00, 0x00, // sfntVersion
        0x00, 0x01, // numTables
        0x00, 0x00, // searchRange
        0x00, 0x00, // entrySelector
        0x00, 0x00, // rangeShift
        // And one partial table directory entry
        0x0b, 0x0a, 0x0d, 0x0d, // tag
    ]);
    let result = SfntFont::get_chunk_positions(&mut reader);
    assert!(result.is_err());
    let err = result.err().unwrap();
    // Should be a "failed to fill whole buffer" error
    assert!(matches!(err, FontIoError::IoError(_)));
    assert_eq!(format!("{err}"), "failed to fill whole buffer");
}

#[test]
fn test_sfnt_font_chunk_reader_valid() {
    let font_bytes = include_bytes!("../../../.devtools/font.otf");
    let mut reader = Cursor::new(font_bytes);
    let result = SfntFont::get_chunk_positions(&mut reader);
    assert!(result.is_ok());
    let mut positions = result.unwrap();
    // Get the first position, should be the header
    let header = positions.first().unwrap();
    assert_eq!(header.offset(), 0);
    assert_eq!(header.length(), 188);
    assert_eq!(header.name(), b" HDR");
    assert_eq!(header.chunk_type(), &SfntChunkType::HeaderDirectory);
    assert!(!header.chunk_type().should_hash());
    positions.remove(0);

    // Find the specialized hea1 table, which contains the exclusion of the
    // checksum adjustment
    let head1 = positions.iter().find(|p| p.name() == b"hea1").unwrap();
    assert_eq!(head1.offset(), 196);
    assert_eq!(head1.length(), 4);
    assert_eq!(head1.chunk_type(), &SfntChunkType::ChecksumAdjustment);
    assert!(!head1.chunk_type().should_hash());

    // All the other positions should be included
    positions.retain(|p| p.name() != b"hea1");
    for position in positions {
        assert_eq!(position.chunk_type(), &SfntChunkType::TableData);
        assert!(position.chunk_type().should_hash());
    }
}

#[test]
fn test_sfnt_font_chunk_reader_with_c2pa() {
    // Load the font data bytes
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = Cursor::new(font_data);
    // Read in the font, so we can add a C2PA record
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    // Build up the C2PA record
    let record = ContentCredentialRecord::builder()
        .with_version(0, 1)
        .with_active_manifest_uri("https://example.com".to_string())
        .with_content_credential(vec![0x00, 0x01, 0x02, 0x03])
        .build()
        .unwrap();
    // Add it to the font stream
    font.add_c2pa_record(record).unwrap();
    // Write the font out to a new writer
    let mut writer = Cursor::new(Vec::new());
    let result = font.write(&mut writer);
    assert!(result.is_ok());
    // Get access to the written data
    let written_data = writer.into_inner();
    // And use it in a reader to read chunk positions
    let mut new_reader = Cursor::new(&written_data);
    let result = SfntFont::get_chunk_positions(&mut new_reader);
    assert!(result.is_ok());
    let positions = result.unwrap();
    let c2pa = positions.iter().find(|p| p.name() == b"C2PA").unwrap();
    assert_eq!(c2pa.offset(), 1388);
    assert_eq!(c2pa.length(), 43);
    assert_eq!(c2pa.chunk_type(), &SfntChunkType::C2paTableData);
    assert!(c2pa.chunk_type().should_hash());
}

#[test]
#[tracing_test::traced_test]
fn test_sfnt_font_chunk_reader_tracing() {
    // Load the font data bytes
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut reader = Cursor::new(font_data);
    // Read in the font, so we can add a C2PA record
    let mut font = SfntFont::from_reader(&mut reader).unwrap();
    // Build up the C2PA record
    let record = ContentCredentialRecord::builder()
        .with_version(0, 1)
        .with_active_manifest_uri("https://example.com".to_string())
        .with_content_credential(vec![0x00, 0x01, 0x02, 0x03])
        .build()
        .unwrap();
    // Add it to the font stream
    font.add_c2pa_record(record).unwrap();
    // Write the font out to a new writer
    let mut writer = Cursor::new(Vec::new());
    let result = font.write(&mut writer);
    assert!(result.is_ok());
    // Get access to the written data
    let written_data = writer.into_inner();
    // And use it in a reader to read chunk positions
    let mut new_reader = Cursor::new(&written_data);
    let _ = SfntFont::get_chunk_positions(&mut new_reader);
    assert!(logs_contain("HeaderDirectory position information added"));
    assert!(logs_contain(
        "C2PA table found, adding positional information"
    ));
    assert!(logs_contain("'head' table found, adding positional information, where excluding the checksum adjustment"));
    assert!(logs_contain("Adding positional information for table data"));
}

#[test]
fn test_sfnt_chunk_type_display() {
    assert_eq!(
        SfntChunkType::HeaderDirectory.to_string(),
        "HeaderDirectory"
    );
    assert_eq!(SfntChunkType::TableData.to_string(), "Table Data");
    assert_eq!(
        SfntChunkType::ChecksumAdjustment.to_string(),
        "Checksum Adjustment"
    );
    assert_eq!(SfntChunkType::C2paTableData.to_string(), "C2PA Table Data");
}

#[cfg(feature = "woff")]
#[test]
fn test_try_from_woff_to_sfnt() {
    // Simulate a WOFF font

    use crate::woff1::font::Woff1Font;
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
    let woff_font =
        Woff1Font::from_reader(&mut Cursor::new(woff_data.clone())).unwrap();
    let sfnt_font_result: Result<SfntFont, _> = woff_font.try_into();
    assert!(sfnt_font_result.is_ok());
    let sfnt_font = sfnt_font_result.unwrap();
    assert_eq!(sfnt_font.header.sfntVersion as u32, 0x4f54544f);
    assert_eq!(sfnt_font.header.num_tables(), 1);
    assert!(sfnt_font.contains_table(&FontTag::new(*b"test")));
    let table = sfnt_font.table(&FontTag::new(*b"test"));
    assert!(table.is_some());
    let table = table.unwrap();
    assert_eq!(table.len(), 4);
}

#[cfg(feature = "woff")]
#[test]
#[tracing_test::traced_test]
fn test_try_from_woff_to_sfnt_with_c2pa() {
    use crate::woff1::font::Woff1Font;
    // Simulate a WOFF font
    let woff_data = vec![
        0x77, 0x4f, 0x46, 0x46, // Signature
        0x4f, 0x54, 0x54, 0x4f, // Flavor
        0x00, 0x00, 0x00, 0x88, // Length (136 bytes)
        0x00, 0x02, 0x00, 0x00, // Number of tables + Reserved
        /* Total sfnt size: 96: (12[sfnt header] + 16*2[table
         * directory entries] + (20+26+2padding)[c2pa] + 4[text]) */
        0x00, 0x00, 0x00, 0x60, // Total sfnt size
        0x00, 0x00, 0x00, 0x00, // Major version + Minor version
        0x00, 0x00, 0x00, 0x00, // Metadata Offset
        0x00, 0x00, 0x00, 0x00, // Metadata Length
        0x00, 0x00, 0x00, 0x00, // Metadata Original Length
        0x00, 0x00, 0x00, 0x00, // Private Offset
        0x00, 0x00, 0x00, 0x00, // Private Length
        0x43, 0x32, 0x50, 0x41, // Directory entry - tag (C2PA)
        0x00, 0x00, 0x00, 0x58, // Directory entry - offset
        0x00, 0x00, 0x00, 0x2e, // Directory entry - comp length
        0x00, 0x00, 0x00, 0x2e, // Directory entry - orig length
        0x56, 0x54, 0x0c, 0x33, // Directory entry - orig checksum
        0x74, 0x65, 0x73, 0x74, // Directory entry - tag (text)
        0x00, 0x00, 0x00, 0x54, // Directory entry - offset
        0x00, 0x00, 0x00, 0x04, // Directory entry - comp length
        0x00, 0x00, 0x00, 0x04, // Directory entry - orig length
        0x40, 0x30, 0x02, 0x01, // Directory entry - orig checksum
        0x04, 0x03, 0x02, 0x01, // 'test' table
        /* 'C2PA' table */
        0x00, 0x00, 0x00, 0x01, // major/minor version
        0x00, 0x00, 0x00, 0x14, // manifest URI offset
        0x00, 0x1a, 0x00, 0x00, // manifest URI length + reserved
        0x00, 0x00, 0x00, 0x00, // content credential offset
        0x00, 0x00, 0x00, 0x00, // content credential length
        /* active manifest URI - http://localhost:3001/c2pa\0\0 */
        0x68, 0x74, 0x74, 0x70, //
        0x3a, 0x2f, 0x2f, 0x6c, //
        0x6f, 0x63, 0x61, 0x6c, //
        0x68, 0x6f, 0x73, 0x74, //
        0x3a, 0x33, 0x30, 0x30, //
        0x31, 0x2f, 0x63, 0x32, //
        0x70, 0x61, 0x00, 0x00, // Last bit with 2 bytes of padding
    ];

    let woff_font =
        Woff1Font::from_reader(&mut Cursor::new(woff_data.clone())).unwrap();
    let sfnt_font_result: Result<SfntFont, _> = woff_font.try_into();
    assert!(sfnt_font_result.is_ok());
    let sfnt_font = sfnt_font_result.unwrap();
    assert_eq!(sfnt_font.header.sfntVersion as u32, 0x4f54544f);
    assert_eq!(sfnt_font.header.num_tables(), 1);
    assert!(sfnt_font.contains_table(&FontTag::new(*b"test")));
    let table = sfnt_font.table(&FontTag::new(*b"test"));
    assert!(table.is_some());
    let table = table.unwrap();
    assert_eq!(table.len(), 4);
    assert!(logs_contain("WOFF C2PA will not be added to SFNT font"));
}

#[cfg(feature = "woff")]
#[test]
#[tracing_test::traced_test]
fn test_try_from_woff_to_sfnt_with_compression() {
    use crate::woff1::font::Woff1Font;
    // Simulate a WOFF font
    let woff_data = vec![
        0x77, 0x4f, 0x46, 0x46, // Signature
        0x4f, 0x54, 0x54, 0x4f, // Flavor
        0x00, 0x00, 0x00, 0x70, // Length (112 bytes)
        0x00, 0x02, 0x00, 0x00, // Number of tables + Reserved
        /* Total sfnt size: 96: (12[sfnt header] + 16*2[table
         * directory entries] + (20+26+2padding)[c2c2] + 4[text]) */
        0x00, 0x00, 0x00, 0x60, // Total sfnt size
        0x00, 0x00, 0x00, 0x00, // Major version + Minor version
        0x00, 0x00, 0x00, 0x00, // Metadata Offset
        0x00, 0x00, 0x00, 0x00, // Metadata Length
        0x00, 0x00, 0x00, 0x00, // Metadata Original Length
        0x00, 0x00, 0x00, 0x00, // Private Offset
        0x00, 0x00, 0x00, 0x00, // Private Length
        0x43, 0x32, 0x43, 0x32, // Directory entry - tag (C2C2)
        0x00, 0x00, 0x00, 0x58, // Directory entry - offset
        0x00, 0x00, 0x00, 0x17, // Directory entry - comp length
        0x00, 0x00, 0x00, 0x2e, // Directory entry - orig length
        0x51, 0x6b, 0x21, 0x35, // Directory entry - orig checksum
        0x74, 0x65, 0x73, 0x74, // Directory entry - tag (text)
        0x00, 0x00, 0x00, 0x54, // Directory entry - offset
        0x00, 0x00, 0x00, 0x04, // Directory entry - comp length
        0x00, 0x00, 0x00, 0x04, // Directory entry - orig length
        0x40, 0x30, 0x02, 0x01, // Directory entry - orig checksum
        0x04, 0x03, 0x02, 0x01, // 'test' table
        /* 'C2C2' compressed table */
        0x78, 0x9c, 0x63, 0x60, // 120, 156, 99, 96
        0x60, 0x60, 0x64, 0x60, // 96, 96, 100, 96
        0x60, 0x10, 0x61, 0x90, // 96, 16, 97, 144
        0x62, 0x80, 0x03, 0x03, // 98, 128, 3, 3
        0x9c, 0x00, 0x00, 0x48, // 156, 0, 0, 72
        0xf7, 0x05, 0x10, 0x00, // 247, 5, 16, 0
    ];

    let woff_font =
        Woff1Font::from_reader(&mut Cursor::new(woff_data.clone())).unwrap();
    let sfnt_font_result: Result<SfntFont, _> = woff_font.try_into();
    assert!(sfnt_font_result.is_ok());
    let sfnt_font = sfnt_font_result.unwrap();
    assert_eq!(sfnt_font.header.sfntVersion as u32, 0x4f54544f);
    assert_eq!(sfnt_font.header.num_tables(), 2);
    assert!(sfnt_font.contains_table(&FontTag::new(*b"test")));
    let table = sfnt_font.table(&FontTag::new(*b"test"));
    assert!(table.is_some());
    let table = table.unwrap();
    assert_eq!(table.len(), 4);
    assert!(!logs_contain("WOFF C2PA will not be added to SFNT font"));
    let entry_selector = 2f64.log2().floor() as u16;
    let search_range = 2_u16.pow(entry_selector as u32) * 16;
    let range_shift = 2 * 16 - search_range;
    let field = sfnt_font.header.searchRange;
    assert_eq!(field, search_range);
    let field = sfnt_font.header.entrySelector;
    assert_eq!(field, entry_selector);
    let field = sfnt_font.header.rangeShift;
    assert_eq!(field, range_shift);
}

#[cfg(feature = "woff")]
#[test]
#[tracing_test::traced_test]
fn test_try_from_woff_to_sfnt_with_no_tables() {
    use crate::woff1::font::Woff1Font;
    // Simulate a WOFF font
    let woff_data = vec![
        0x77, 0x4f, 0x46, 0x46, // Signature
        0x4f, 0x54, 0x54, 0x4f, // Flavor
        0x00, 0x00, 0x00, 0x30, // Length
        0x00, 0x00, 0x00, 0x00, // Number of tables + Reserved
        0x00, 0x00, 0x00, 0x18, // Total sfnt size
        0x00, 0x00, 0x00, 0x00, // Major version + Minor version
        0x00, 0x00, 0x00, 0x2c, // Metadata Offset
        0x00, 0x00, 0x00, 0x04, // Metadata Length
        0x00, 0x00, 0x00, 0x04, // Metadata Original Length
        0x00, 0x00, 0x00, 0x00, // Private Offset
        0x00, 0x00, 0x00, 0x00, // Private Length
        0x77, 0x55, 0x33, 0x58, // Metadata
    ];

    let woff_font =
        Woff1Font::from_reader(&mut Cursor::new(woff_data.clone())).unwrap();
    let sfnt_font_result: Result<SfntFont, _> = woff_font.try_into();
    assert!(sfnt_font_result.is_err());
    assert!(matches!(sfnt_font_result, Err(FontIoError::NoTablesFound)));
}
