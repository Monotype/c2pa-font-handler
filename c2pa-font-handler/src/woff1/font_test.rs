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

use std::{
    collections::btree_map::Entry,
    io::{Cursor, Write},
};

use super::Woff1Font;
use crate::{
    c2pa::{
        C2PASupport, ContentCredentialRecordBuilder, UpdatableC2PA,
        UpdateContentCredentialRecord,
    },
    chunks::{ChunkReader, ChunkTypeTrait},
    data::Data,
    error::FontIoError,
    magic::Magic,
    tag::FontTag,
    woff1::{font::WoffChunkType, header::Woff1Header, table::NamedTable},
    Font, FontDataRead, FontDirectory, FontTable, MutFontDataWrite,
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
    assert!(woff.contains_table(&FontTag::HEAD));
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
            signature: Magic::Woff,
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
    let private_data = woff.private_data.unwrap();
    assert_eq!(private_data.len(), 4);
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
            signature: Magic::Woff,
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
    let metadata = woff.metadata.unwrap();
    assert_eq!(metadata.len(), 4);
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
    let private_data = woff.private_data.unwrap();
    assert_eq!(private_data.len(), 4);
}

#[test]
fn test_woff_font_chunk_reader_bad_header() {
    let mut reader = std::io::Cursor::new(vec![0u8; 10]);
    let result = Woff1Font::get_chunk_positions(&mut reader);
    assert!(result.is_err());
    let err = result.err().unwrap();
    // Since we didn't do an appropriate magic for the font, we should get an
    // unknown magic error.
    assert!(matches!(err, FontIoError::UnknownMagic(_)));
}

#[test]
fn test_woff_font_chunk_reader_bad_directory() {
    // Mimic a bad font in memory, where the directory is too short
    let mut reader = std::io::Cursor::new(vec![
        // Simulate the magic number
        0x77, 0x4f, 0x46, 0x46, // sfntVersion
        0x45, 0x54, 0x54, 0x4f, // flavor
        0x00, 0x00, 0x00, 0x48, // length
        0x00, 0x01, // numTables
        0x00, 0x00, // reserved
        0x00, 0x00, 0x00, 0x18, // totalSfntSize
        0x00, 0x00, // majorVersion
        0x00, 0x00, // minorVersion
        0x00, 0x00, 0x00, 0x00, // metaOffset
        0x00, 0x00, 0x00, 0x00, // metaLength
        0x00, 0x00, 0x00, 0x00, // metaOrigLength
        0x00, 0x00, 0x00, 0x00, // privOffset
        0x00, 0x00, 0x00, 0x00, // privLength
        // And one partial table directory entry
        0x0b, 0x0a, 0x0d, 0x0d, // tag
    ]);
    let result = Woff1Font::get_chunk_positions(&mut reader);
    assert!(result.is_err());
    let err = result.err().unwrap();
    // Should be a "failed to fill whole buffer" error
    assert!(matches!(err, FontIoError::IoError(_)));
    assert_eq!(format!("{err}"), "failed to fill whole buffer");
}

#[test]
fn test_woff_font_chunk_reader_valid() {
    let font_bytes = include_bytes!("../../../.devtools/font.woff");
    let mut reader = std::io::Cursor::new(font_bytes);
    let result = Woff1Font::get_chunk_positions(&mut reader);
    assert!(result.is_ok());
    let mut positions = result.unwrap();
    // Get the first position, should be the header
    let header = positions.first().unwrap();
    assert_eq!(header.offset(), 0);
    assert_eq!(header.length(), Woff1Header::SIZE);
    assert_eq!(header.name(), b"\x00\x00\x00W");
    assert_eq!(header.chunk_type(), &WoffChunkType::Header);
    assert!(header.chunk_type().should_hash());
    positions.remove(0);

    // Then the 2nd one should be the directory
    let directory = positions.first().unwrap();
    assert_eq!(directory.offset(), Woff1Header::SIZE);
    assert_eq!(directory.length(), 200);
    assert_eq!(directory.name(), b"\x00\x00\x01D");
    assert_eq!(directory.chunk_type(), &WoffChunkType::DirectoryEntry);
    assert!(directory.chunk_type().should_hash());
    positions.remove(0);

    // Other positions should be included
    for position in positions {
        assert_eq!(position.chunk_type(), &WoffChunkType::TableData);
        assert!(position.chunk_type().should_hash());
    }
}

#[test]
fn test_woff_font_chunk_reader_metadata_private() {
    // Read in the font bytes
    let font_bytes = include_bytes!("../../../.devtools/font.woff");
    let mut reader = std::io::Cursor::new(font_bytes);
    // Parse into a WOFF font container
    let mut font = Woff1Font::from_reader(&mut reader).unwrap();
    // Set the metadata and private data
    font.metadata = Some(Data::new(vec![0x01, 0x02, 0x03, 0x04]));
    font.private_data = Some(Data::new(vec![0x05, 0x06, 0x07, 0x08]));
    // And setup to write it back to a buffer
    let mut writer = std::io::Cursor::new(Vec::new());
    font.write(&mut writer).unwrap();
    // Create a new reader around the new written data
    let mut reader = std::io::Cursor::new(writer.into_inner());
    // And use that reader to get the positions
    let result = Woff1Font::get_chunk_positions(&mut reader);
    assert!(result.is_ok());
    let positions = result.unwrap();
    // Should be able to find the metadata, which should be hashed
    let metadata = positions
        .iter()
        .find(|p| p.name() == b"\x7F\x7F\x7Fm")
        .unwrap();
    assert_eq!(metadata.offset(), 884);
    assert_eq!(metadata.length(), 4);
    assert_eq!(metadata.chunk_type(), &WoffChunkType::Metadata);
    assert!(metadata.chunk_type().should_hash());
    // And should be able to find the private data, which should NOT be hashed??
    let private = positions
        .iter()
        .find(|p| p.name() == b"\x7F\x7F\x7FP")
        .unwrap();
    assert_eq!(private.offset(), 888);
    assert_eq!(private.length(), 4);
    assert_eq!(private.chunk_type(), &WoffChunkType::Private);
    assert!(!private.chunk_type().should_hash());
}

#[test]
#[tracing_test::traced_test]
fn test_woff_font_chunk_reader_tracing() {
    // Load the font data bytes
    let font_data = include_bytes!("../../../.devtools/font.woff");
    let mut reader = std::io::Cursor::new(font_data);
    let _ = Woff1Font::get_chunk_positions(&mut reader);
    assert!(logs_contain("Header position information added"));
    assert!(logs_contain("Directory position information added"));
    assert!(logs_contain("Table data position information added"));
    assert!(!logs_contain("Metadata position information added"));
    assert!(!logs_contain("Private data position information added"));
}

#[test]
fn test_woff_chunk_type_display() {
    assert_eq!(format!("{}", WoffChunkType::Header), "Header");
    assert_eq!(
        format!("{}", WoffChunkType::DirectoryEntry),
        "Directory Entry"
    );
    assert_eq!(format!("{}", WoffChunkType::TableData), "Table Data");
    assert_eq!(format!("{}", WoffChunkType::Metadata), "Metadata");
    assert_eq!(format!("{}", WoffChunkType::Private), "Private Data");
}

#[test]
#[tracing_test::traced_test]
fn test_woff_add_c2pa_record() {
    // Load the font data bytes
    let font_data = include_bytes!("../../../.devtools/font.woff");
    let mut reader = std::io::Cursor::new(font_data);
    let mut woff = Woff1Font::from_reader(&mut reader).unwrap();
    // Add a C2PA record to the font
    let c2pa_record = ContentCredentialRecordBuilder::default()
        .with_active_manifest_uri(
            "https://example.com/manifest.json".to_string(),
        )
        .with_content_credential(b"example-credential-with-some-sample-data-which should cause the compression path to take over".to_vec())
        .build()
        .unwrap();
    woff.add_c2pa_record(c2pa_record).unwrap();
    // Check that the C2PA record was added successfully
    assert!(woff.has_c2pa());
    // Temporarily write the font to a temporary file to test reading back in
    let mut writer = std::io::Cursor::new(Vec::new());
    woff.write(&mut writer).unwrap();
    writer.flush().unwrap();
    let woff_data = writer.into_inner();
    let mut reader = std::io::Cursor::new(woff_data);
    let woff = Woff1Font::from_reader(&mut reader).unwrap();
    // Check that the C2PA record is still present
    assert!(woff.has_c2pa());
    let record = woff.get_c2pa().unwrap().unwrap();
    assert_eq!(
        record.active_manifest_uri(),
        Some("https://example.com/manifest.json")
    );
    assert!(logs_contain("Compressing C2PA table; saved 27 bytes"));
}

#[test]
#[tracing_test::traced_test]
fn test_woff_update_c2pa_record_replaces_table() {
    // Load the font data bytes
    let font_data = include_bytes!("../../../.devtools/font_with_c2pa.woff");
    let mut reader = std::io::Cursor::new(font_data);
    let mut woff = Woff1Font::from_reader(&mut reader).unwrap();
    // Check that the C2PA record was added successfully
    assert!(woff.has_c2pa());
    // Update the C2PA record
    let updated_c2pa_record = UpdateContentCredentialRecord::builder()
        .with_active_manifest_uri(
            "https://example.com/updated_manifest.json".to_string(),
        )
        .build();
    woff.update_c2pa_record(updated_c2pa_record).unwrap();
    // Make sure we are not leaving the older table around too, but replacing it
    let count = woff.tables.iter().filter(|t| t.0 == &FontTag::C2PA).count();
    assert_eq!(count, 1);
    // Check that the C2PA record was updated successfully
    assert!(woff.has_c2pa());
}

#[test]
#[tracing_test::traced_test]
fn test_woff_empty_table_should_not_compress() {
    // Load the font data bytes
    let font_data = include_bytes!("../../../.devtools/font.woff");
    let mut reader = std::io::Cursor::new(font_data);
    let mut woff = Woff1Font::from_reader(&mut reader).unwrap();
    let c2pa_record = ContentCredentialRecordBuilder::default()
        .with_active_manifest_uri(
            "https://example.com/manifest.json".to_string(),
        )
        .build()
        .unwrap();
    woff.add_c2pa_record(c2pa_record).unwrap();
    // Check that the C2PA record was added successfully
    assert!(woff.has_c2pa());
    // Temporarily write the font to a temporary file to test reading back in
    let mut writer = std::io::Cursor::new(Vec::new());
    woff.write(&mut writer).unwrap();
    writer.flush().unwrap();
    let woff_data = writer.into_inner();
    let mut reader = std::io::Cursor::new(woff_data);
    let woff = Woff1Font::from_reader(&mut reader).unwrap();
    // Check that the C2PA record is still present
    assert!(woff.has_c2pa());
    let record = woff.get_c2pa().unwrap().unwrap();
    assert_eq!(
        record.active_manifest_uri(),
        Some("https://example.com/manifest.json")
    );
    assert!(logs_contain("Not compressing C2PA table"))
}

#[test]
#[tracing_test::traced_test]
fn test_remove_c2pa_record_for_woff() {
    // Load the font data bytes
    let font_data = include_bytes!("../../../.devtools/font.woff");
    let mut reader = std::io::Cursor::new(font_data);
    let mut woff = Woff1Font::from_reader(&mut reader).unwrap();
    // Add a C2PA record to the font
    let c2pa_record = ContentCredentialRecordBuilder::default()
        .with_active_manifest_uri(
            "https://example.com/manifest.json".to_string(),
        )
        .build()
        .unwrap();
    woff.add_c2pa_record(c2pa_record).unwrap();
    // Check that the C2PA record was added successfully
    assert!(woff.has_c2pa());
    // Remove the C2PA record
    woff.remove_c2pa_record().unwrap();
    // Check that the C2PA record was removed successfully
    assert!(!woff.has_c2pa());
}

#[test]
fn test_get_c2pa_for_unsigned_woff() {
    // Load the font data bytes
    let font_data = include_bytes!("../../../.devtools/font.woff");
    let mut reader = std::io::Cursor::new(font_data);
    let woff = Woff1Font::from_reader(&mut reader).unwrap();
    // Check that the C2PA record is not present
    assert!(woff.get_c2pa().unwrap().is_none());
}

#[test]
fn test_remove_c2pa_on_unsigned_woff() {
    // Load the font data bytes
    let font_data = include_bytes!("../../../.devtools/font.woff");
    let mut reader = std::io::Cursor::new(font_data);
    let mut woff = Woff1Font::from_reader(&mut reader).unwrap();
    // Check that the C2PA record is not present
    assert!(!woff.has_c2pa());
    // Attempt to remove the C2PA record
    let result = woff.remove_c2pa_record();
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(FontIoError::ContentCredentialNotFound)
    ));
}

#[test]
fn test_add_c2pa_when_one_is_already_present() {
    // Load the font data bytes
    let font_data = include_bytes!("../../../.devtools/font.woff");
    let mut reader = std::io::Cursor::new(font_data);
    let mut woff = Woff1Font::from_reader(&mut reader).unwrap();
    // Add a C2PA record to the font
    let c2pa_record = ContentCredentialRecordBuilder::default()
        .with_active_manifest_uri(
            "https://example.com/manifest.json".to_string(),
        )
        .build()
        .unwrap();
    woff.add_c2pa_record(c2pa_record.clone()).unwrap();
    // Check that the C2PA record was added successfully
    assert!(woff.has_c2pa());
    let result = woff.add_c2pa_record(c2pa_record);
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(FontIoError::ContentCredentialAlreadyExists)
    ));
}

#[test]
#[tracing_test::traced_test]
fn test_get_c2pa_from_woff_font() {
    // Load the signed WOFF font data bytes
    let font_data = include_bytes!("../../../.devtools/font_with_c2pa.woff");
    let mut reader = std::io::Cursor::new(font_data);
    let woff = Woff1Font::from_reader(&mut reader).unwrap();
    // Check that the C2PA record was added successfully
    assert!(woff.has_c2pa());
    let result = woff.get_c2pa();
    assert!(result.is_ok());
    let c2pa_record = result.unwrap().unwrap();
    assert_eq!(
        c2pa_record.active_manifest_uri(),
        Some("https://example.com/manifest.json")
    );
}

#[test]
#[tracing_test::traced_test]
fn test_update_c2pa_record_when_not_present() {
    // Load the font data bytes
    let font_data = include_bytes!("../../../.devtools/font.woff");
    let mut reader = std::io::Cursor::new(font_data);
    let mut woff = Woff1Font::from_reader(&mut reader).unwrap();
    // Add a C2PA record to the font
    let c2pa_record = UpdateContentCredentialRecord::builder()
        .with_active_manifest_uri(
            "https://example.com/manifest.json".to_string(),
        )
        .build();
    // Attempt to update the C2PA record
    let result = woff.update_c2pa_record(c2pa_record);
    assert!(result.is_ok());
    assert!(logs_contain("Adding C2PA table"));
}

#[test]
#[tracing_test::traced_test]
fn test_update_c2pa_record() {
    // Load the signed WOFF font data bytes
    let font_data = include_bytes!("../../../.devtools/font_with_c2pa.woff");
    let mut reader = std::io::Cursor::new(font_data);
    let mut woff = Woff1Font::from_reader(&mut reader).unwrap();
    // Check that the C2PA record was added successfully
    assert!(woff.has_c2pa());
    // Update the C2PA record
    let c2pa_record = UpdateContentCredentialRecord::builder()
        .with_active_manifest_uri(
            "https://example.com/manifest.json".to_string(),
        )
        .build();
    let result = woff.update_c2pa_record(c2pa_record);
    assert!(result.is_ok());
    assert!(logs_contain("Updating C2PA table"));
}

#[test]
#[tracing_test::traced_test]
fn test_update_c2pa_record_invalid_c2pa_table() {
    // Load the signed WOFF font data bytes
    let font_data = include_bytes!("../../../.devtools/font_with_c2pa.woff");
    let mut reader = std::io::Cursor::new(font_data);
    let mut woff = Woff1Font::from_reader(&mut reader).unwrap();

    // Replace the C2PA table with an invalid table type, which is just a
    // Generic
    match woff.tables.entry(FontTag::C2PA) {
        Entry::Occupied(mut entry) => {
            // We have to go out of our way to make sure the C2PA table is
            // listed as a Generic table, which is not the default.
            entry.insert(NamedTable::Generic(Data::new(vec![
                0x00, 0x00, 0x00, 0x00,
            ])));
        }
        _ => panic!("C2PA table not found"),
    }

    // Update the C2PA record with an invalid table
    let c2pa_record = UpdateContentCredentialRecord::builder()
        .with_active_manifest_uri(
            "https://example.com/manifest.json".to_string(),
        )
        .build();
    // Attempt to update the C2PA record
    let result = woff.update_c2pa_record(c2pa_record);
    // Which should result in an error
    assert!(result.is_err());
    let err = result.err().unwrap();
    // And that error should be a "Invalid C2PA table" error
    assert!(matches!(err, FontIoError::InvalidC2paTableContainer));
    assert!(logs_contain("C2PA tag exists but is not a C2PA table"));
}
