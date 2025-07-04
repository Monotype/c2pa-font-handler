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

//! Tests for the 'C2PA' SFNT table module

use std::io::Cursor;

use super::*;
use crate::c2pa::UpdateContentCredentialRecord;

#[test]
fn test_from_reader_table_c2pa_raw() {
    // Create C2PA table entry data
    let mut data = vec![];
    data.extend_from_slice(&[0x00, 0x01]); // major_version
    data.extend_from_slice(&[0x00, 0x04]); // minor_version
    data.extend_from_slice(&[0x00, 0x00, 0x01, 0x90]); // active manifest uri offset
    data.extend_from_slice(&[0x01, 0x2c]); // active manifest uri length
    data.extend_from_slice(&[0x00, 0x00]); // reserved
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x14]); // content_credential offset
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]); // content_credential length
    data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // content_credential
                                                       // create a cursor/reader around the data
    let mut reader = Cursor::new(data);
    let result = TableC2PARaw::from_reader(&mut reader);
    assert!(result.is_ok());
    let table = result.unwrap();
    let major_version = table.majorVersion;
    assert_eq!(major_version, 1);
    let minor_version = table.minorVersion;
    assert_eq!(minor_version, 4);
    let active_manifest_uri_offset = table.activeManifestUriOffset;
    assert_eq!(active_manifest_uri_offset, 400);
    let active_manifest_uri_length = table.activeManifestUriLength;
    assert_eq!(active_manifest_uri_length, 300);
    let content_credential_offset = table.manifestStoreOffset;
    assert_eq!(content_credential_offset, 20);
    let content_credential_length = table.manifestStoreLength;
    assert_eq!(content_credential_length, 4);
}

#[test]
fn test_from_reader_table_c2pa_raw_not_enough_data() {
    // Create C2PA table entry data
    let mut data = vec![];
    data.extend_from_slice(&[0x00, 0x01]); // major_version
    let mut reader = Cursor::new(data);
    let result = TableC2PARaw::from_reader(&mut reader);
    assert!(result.is_err());
    let error = result.err().unwrap();
    assert!(matches!(error, FontIoError::IoError(_)));
}

#[test]
fn test_write_table_c2pa_raw() {
    let table = TableC2PARaw {
        majorVersion: 1,
        minorVersion: 4,
        activeManifestUriOffset: 400,
        activeManifestUriLength: 300,
        reserved: 0,
        manifestStoreOffset: 20,
        manifestStoreLength: 4,
    };
    let mut data = vec![];
    let result = table.write(&mut data);
    assert!(result.is_ok());
    let expected_data = vec![
        0x00, 0x01, // major_version
        0x00, 0x04, // minor_version
        0x00, 0x00, 0x01, 0x90, // active manifest uri offset
        0x01, 0x2c, // active manifest uri length
        0x00, 0x00, // reserved
        0x00, 0x00, 0x00, 0x14, // content_credential offset
        0x00, 0x00, 0x00, 0x04, // content_credential length
    ];
    assert_eq!(data, expected_data);
}

#[test]
fn test_table_c2pa_raw_checksum() {
    let table = TableC2PARaw {
        majorVersion: 1,
        minorVersion: 4,
        activeManifestUriOffset: 400,
        activeManifestUriLength: 300,
        reserved: 0,
        manifestStoreOffset: 20,
        manifestStoreLength: 4,
    };
    let checksum = table.checksum();
    let mut expected_checksum = Wrapping(0x00000000);
    // Make 32-bit big-endian checksum
    // Shift the major version by 16 bits to the left and add the minor version
    expected_checksum += Wrapping(65536 + 4);
    expected_checksum += Wrapping(400);
    // Shift the active manifest uri length by 16 bits to the left and add the
    // reserved field
    expected_checksum += Wrapping(300 * 65536);
    expected_checksum += Wrapping(20);
    expected_checksum += Wrapping(4);
    assert_eq!(checksum, expected_checksum);
}

#[test]
fn test_table_c2pa_raw_len() {
    let table = TableC2PARaw {
        majorVersion: 1,
        minorVersion: 4,
        activeManifestUriOffset: 400,
        activeManifestUriLength: 300,
        reserved: 0,
        manifestStoreOffset: 20,
        manifestStoreLength: 4,
    };
    let len = table.len();
    assert_eq!(len, 20);
}

#[test]
fn test_table_c2pa_from_reader() {
    // Create C2PA table entry data
    let mut data = vec![];
    data.extend_from_slice(&[0x00, 0x01]); // major_version
    data.extend_from_slice(&[0x00, 0x04]); // minor_version
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x18]); // active manifest uri offset
    data.extend_from_slice(&[0x00, 0x04]); // active manifest uri length
    data.extend_from_slice(&[0x00, 0x00]); // reserved
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x14]); // content_credential offset
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]); // content_credential length
    data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // content_credential
    data.extend_from_slice(b"test"); // active content uri
                                     // create a cursor/reader around the data
    let mut reader = Cursor::new(data);
    let result = TableC2PA::from_reader_exact(&mut reader, 0, 28);
    assert!(result.is_ok());
    let table = result.unwrap();
    let major_version = table.major_version;
    assert_eq!(major_version, 1);
    let minor_version = table.minor_version;
    assert_eq!(minor_version, 4);
    let active_manifest_uri_offset = table.active_manifest_uri;
    assert_eq!(active_manifest_uri_offset, Some("test".to_string()));
    let content_credential_offset = table.manifest_store;
    assert_eq!(content_credential_offset, Some(vec![1, 2, 3, 4]));
}

#[test]
fn test_table_c2pa_from_reader_invalid_active_uri_offset() {
    // Create C2PA table entry data
    let mut data = vec![];
    data.extend_from_slice(&[0x00, 0x01]); // major_version
    data.extend_from_slice(&[0x00, 0x04]); // minor_version
    data.extend_from_slice(&[0x00, 0x00, 0x10, 0x18]); // active manifest uri offset
    data.extend_from_slice(&[0x00, 0x04]); // active manifest uri length
    data.extend_from_slice(&[0x00, 0x00]); // reserved
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x14]); // content_credential offset
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]); // content_credential length
    data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // content_credential
    data.extend_from_slice(b"test"); // active content uri
                                     // create a cursor/reader around the data
    let mut reader = Cursor::new(data);
    let result = TableC2PA::from_reader_exact(&mut reader, 0, 28);
    assert!(result.is_err());
    let error = result.err().unwrap();
    assert!(matches!(
        error,
        FontIoError::LoadTableTruncated(FontTag::C2PA)
    ));
}

#[test]
fn test_table_c2pa_from_reader_invalid_manifest_offset() {
    // Create C2PA table entry data
    let mut data = vec![];
    data.extend_from_slice(&[0x00, 0x01]); // major_version
    data.extend_from_slice(&[0x00, 0x04]); // minor_version
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // active manifest uri offset
    data.extend_from_slice(&[0x00, 0x00]); // active manifest uri length
    data.extend_from_slice(&[0x00, 0x00]); // reserved
    data.extend_from_slice(&[0x00, 0x00, 0x10, 0x14]); // content_credential offset
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]); // content_credential length
    let len = data.len();
    // create a cursor/reader around the data
    let mut reader = Cursor::new(data);
    let result = TableC2PA::from_reader_exact(&mut reader, 0, len);
    assert!(result.is_err());
    let error = result.err().unwrap();
    assert!(matches!(
        error,
        FontIoError::LoadTableTruncated(FontTag::C2PA)
    ));
}

#[test]
fn test_table_c2pa_from_reader_with_no_data() {
    // Create C2PA table entry data
    let mut data = vec![];
    data.extend_from_slice(&[0x00, 0x01]); // major_version
    data.extend_from_slice(&[0x00, 0x04]); // minor_version
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // active manifest uri offset
    data.extend_from_slice(&[0x00, 0x00]); // active manifest uri length
    data.extend_from_slice(&[0x00, 0x00]); // reserved
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // content_credential offset
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // content_credential length
    let len = data.len();
    // create a cursor/reader around the data
    let mut reader = Cursor::new(data);
    let result = TableC2PA::from_reader_exact(&mut reader, 0, len);
    assert!(result.is_ok());
    let table = result.unwrap();
    let major_version = table.major_version;
    assert_eq!(major_version, 1);
    let minor_version = table.minor_version;
    assert_eq!(minor_version, 4);
    let active_manifest_uri_offset = table.active_manifest_uri;
    assert_eq!(active_manifest_uri_offset, None);
    let content_credential_offset = table.manifest_store;
    assert_eq!(content_credential_offset, None);
}

#[test]
fn test_table_c2pa_write() {
    let table = TableC2PA {
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: Some("test".to_string()),
        manifest_store: Some(vec![1, 2, 3, 4]),
    };
    let mut data = vec![];
    let result = table.write(&mut data);
    assert!(result.is_ok());
    let expected_data = vec![
        0x00, 0x01, // major_version
        0x00, 0x04, // minor_version
        0x00, 0x00, 0x00, 0x14, // active manifest uri offset
        0x00, 0x04, // active manifest uri length
        0x00, 0x00, // reserved
        0x00, 0x00, 0x00, 0x18, // content_credential offset
        0x00, 0x00, 0x00, 0x04, // content_credential length
        b't', b'e', b's', b't', // active content uri
        0x01, 0x02, 0x03, 0x04, // content_credential
    ];
    assert_eq!(data, expected_data);
}

#[test]
fn test_table_c2pa_write_no_data() {
    let table = TableC2PA {
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: None,
        manifest_store: None,
    };
    let mut data = vec![];
    let result = table.write(&mut data);
    assert!(result.is_ok());
    let expected_data = vec![
        0x00, 0x01, // major_version
        0x00, 0x04, // minor_version
        0x00, 0x00, 0x00, 0x00, // active manifest uri offset
        0x00, 0x00, // active manifest uri length
        0x00, 0x00, // reserved
        0x00, 0x00, 0x00, 0x00, // content_credential offset
        0x00, 0x00, 0x00, 0x00, // content_credential length
    ];
    assert_eq!(data, expected_data);
}

#[test]
fn test_table_c2pa_checksum() {
    let table = TableC2PA {
        major_version: 0,
        minor_version: 1,
        active_manifest_uri: Some("test".to_string()),
        manifest_store: Some(vec![1, 2, 3, 4]),
    };
    let checksum = table.checksum();
    // Start with the major + minor version
    let mut expected_checksum = Wrapping(0x00000001);
    // Then calculate the active manifest offset from the header
    expected_checksum += Wrapping(0x00000014);
    // And the active manifest uri length with the reserved field
    expected_checksum += Wrapping(0x00040000);
    // And now the offset to the content credential
    expected_checksum += Wrapping(0x00000018);
    // And the length of the content credential
    expected_checksum += Wrapping(0x00000004);
    // The first 4 bytes of the active manifest uri
    expected_checksum += Wrapping(0x74657374);
    // The content credential
    expected_checksum += Wrapping(0x01020304);

    assert_eq!(checksum, expected_checksum);
}

/// Test the checksum calculation when the active manifest uri is not 4-byte
/// aligned. This was reported as an issue in the original code, so test case
/// added to ensure it is fixed.
#[test]
fn test_table_c2pa_checksum_non_4_byte_aligned() {
    let table = TableC2PA {
        major_version: 0,
        minor_version: 1,
        active_manifest_uri: Some("test1".to_string()),
        manifest_store: Some(vec![1, 2, 3, 4]),
    };
    let checksum = table.checksum();
    // Start with the major + minor version
    let mut expected_checksum = Wrapping(0x00000001);
    // Then calculate the active manifest offset from the header
    expected_checksum += Wrapping(0x00000014);
    // And the active manifest uri length with the reserved field
    expected_checksum += Wrapping(0x00050000);
    // And now the offset to the content credential
    expected_checksum += Wrapping(0x00000019);
    // And the length of the content credential
    expected_checksum += Wrapping(0x00000004);
    // The first 4 bytes of the active manifest uri
    expected_checksum += Wrapping(0x74657374);
    // And the last '1' with padding + the first 3 bytes of the content
    // credential
    expected_checksum += Wrapping(0x31010203);
    // The last byte of the content credential padded
    expected_checksum += Wrapping(0x04000000);

    assert_eq!(checksum, expected_checksum);
}

#[test]
fn test_table_c2pa_checksum_with_no_data() {
    let table = TableC2PA {
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: None,
        manifest_store: None,
    };
    let checksum = table.checksum();
    let mut expected_checksum = Wrapping(0x00000000);
    // Make 32-bit big-endian checksum
    // Shift the major version by 16 bits to the left and add the minor version
    expected_checksum += Wrapping(65536 + 4);

    assert_eq!(checksum, expected_checksum);
}

#[test]
fn test_table_c2pa_len() {
    let table = TableC2PA {
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: Some("test".to_string()),
        manifest_store: Some(vec![1, 2, 3, 4]),
    };
    let len = table.len();
    assert_eq!(len, 28);
}

#[test]
fn test_table_c2pa_raw_from_table_c2pa() {
    let table = TableC2PA {
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: Some("test".to_string()),
        manifest_store: Some(vec![1, 2, 3, 4]),
    };
    let result = TableC2PARaw::from_table(&table);
    assert!(result.is_ok());
    let table_raw = result.unwrap();
    let major_version = table_raw.majorVersion;
    assert_eq!(major_version, 1);
    let minor_version = table_raw.minorVersion;
    assert_eq!(minor_version, 4);
    let active_manifest_uri_offset = table_raw.activeManifestUriOffset;
    assert_eq!(active_manifest_uri_offset, 20);
    let active_manifest_uri_length = table_raw.activeManifestUriLength;
    assert_eq!(active_manifest_uri_length, 4);
    let reserved = table_raw.reserved;
    assert_eq!(reserved, 0);
    let content_credential_offset = table_raw.manifestStoreOffset;
    assert_eq!(content_credential_offset, 24);
    let content_credential_length = table_raw.manifestStoreLength;
    assert_eq!(content_credential_length, 4);
}

#[test]
fn test_table_c2pa_raw_from_table_c2pa_with_no_data() {
    let table = TableC2PA {
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: None,
        manifest_store: None,
    };
    let result = TableC2PARaw::from_table(&table);
    assert!(result.is_ok());
    let table_raw = result.unwrap();
    let major_version = table_raw.majorVersion;
    assert_eq!(major_version, 1);
    let minor_version = table_raw.minorVersion;
    assert_eq!(minor_version, 4);
    let active_manifest_uri_offset = table_raw.activeManifestUriOffset;
    assert_eq!(active_manifest_uri_offset, 0);
    let active_manifest_uri_length = table_raw.activeManifestUriLength;
    assert_eq!(active_manifest_uri_length, 0);
    let reserved = table_raw.reserved;
    assert_eq!(reserved, 0);
    let content_credential_offset = table_raw.manifestStoreOffset;
    assert_eq!(content_credential_offset, 0);
    let content_credential_length = table_raw.manifestStoreLength;
    assert_eq!(content_credential_length, 0);
}

#[test]
fn test_table_c2pa_raw_from_table_c2pa_with_no_uri() {
    let table = TableC2PA {
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: None,
        manifest_store: Some(vec![1, 2, 3, 4]),
    };
    let result = TableC2PARaw::from_table(&table);
    assert!(result.is_ok());
    let table_raw = result.unwrap();
    let major_version = table_raw.majorVersion;
    assert_eq!(major_version, 1);
    let minor_version = table_raw.minorVersion;
    assert_eq!(minor_version, 4);
    let active_manifest_uri_offset = table_raw.activeManifestUriOffset;
    assert_eq!(active_manifest_uri_offset, 0);
    let active_manifest_uri_length = table_raw.activeManifestUriLength;
    assert_eq!(active_manifest_uri_length, 0);
    let reserved = table_raw.reserved;
    assert_eq!(reserved, 0);
    let content_credential_offset = table_raw.manifestStoreOffset;
    assert_eq!(content_credential_offset, 20);
    let content_credential_length = table_raw.manifestStoreLength;
    assert_eq!(content_credential_length, 4);
}

#[test]
fn test_table_c2pa_update_remove_uri() {
    let mut table = TableC2PA {
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: Some("test".to_string()),
        manifest_store: Some(vec![1, 2, 3, 4]),
    };
    let update_record = UpdateContentCredentialRecord::builder()
        .without_active_manifest_uri()
        .build();
    _ = table.update_c2pa_record(update_record);
    assert_eq!(table.active_manifest_uri, None);
    assert_eq!(table.manifest_store, Some(vec![1, 2, 3, 4]));
}

#[test]
fn test_table_c2pa_update_with_new_uri() {
    let mut table = TableC2PA {
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: Some("test".to_string()),
        manifest_store: Some(vec![1, 2, 3, 4]),
    };
    let update_record = UpdateContentCredentialRecord::builder()
        .with_active_manifest_uri("new_test".to_string())
        .build();
    _ = table.update_c2pa_record(update_record);
    assert_eq!(table.active_manifest_uri, Some("new_test".to_string()));
    assert_eq!(table.manifest_store, Some(vec![1, 2, 3, 4]));
}

#[test]
fn test_table_c2pa_update_remove_manifest() {
    let mut table = TableC2PA {
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: Some("test".to_string()),
        manifest_store: Some(vec![1, 2, 3, 4]),
    };
    let update_record = UpdateContentCredentialRecord::builder()
        .without_content_credentials()
        .build();
    _ = table.update_c2pa_record(update_record);
    assert_eq!(table.active_manifest_uri, Some("test".to_string()));
    assert_eq!(table.manifest_store, None);
}

#[test]
fn test_table_c2pa_update_with_new_manifest() {
    let mut table = TableC2PA {
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: Some("test".to_string()),
        manifest_store: Some(vec![1, 2, 3, 4]),
    };
    let update_record = UpdateContentCredentialRecord::builder()
        .with_content_credential(vec![5, 6, 7, 8])
        .build();
    _ = table.update_c2pa_record(update_record);
    assert_eq!(table.active_manifest_uri, Some("test".to_string()));
    assert_eq!(table.manifest_store, Some(vec![5, 6, 7, 8]));
}

#[test]
fn test_table_c2pa_read_exact_less_than_minimum() {
    // There is enough data to read
    let mut data = vec![];
    data.extend_from_slice(&[0x00, 0x01]); // major_version
    data.extend_from_slice(&[0x00, 0x04]); // minor_version
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // active manifest uri offset
    data.extend_from_slice(&[0x00, 0x00]); // active manifest uri length
    data.extend_from_slice(&[0x00, 0x00]); // reserved
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // content_credential offset
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // content_credential length
    let mut reader = Cursor::new(data);
    // But when calling the method we don't tell it to read enough data to
    // capture the table
    let result = TableC2PA::from_reader_exact(&mut reader, 0, 0);
    assert!(result.is_err());
    let error = result.err().unwrap();
    assert!(matches!(
        error,
        FontIoError::LoadTableTruncated(FontTag::C2PA)
    ));
}

#[test]
fn test_table_c2pa_read_exact_fails_to_read_raw() {
    // There is enough data to read
    let mut data = vec![];
    data.extend_from_slice(&[0x00, 0x01]); // major_version
    data.extend_from_slice(&[0x00, 0x04]); // minor_version
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // active manifest uri offset
    data.extend_from_slice(&[0x00, 0x00]); // active manifest uri length
    data.extend_from_slice(&[0x00, 0x00]); // reserved
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // content_credential offset
                                                       //data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // content_credential
                                                       // length
    let mut reader = Cursor::new(data);
    // But when calling the method we don't tell it to read enough data to
    // capture the table
    let result = TableC2PA::from_reader_exact(
        &mut reader,
        0,
        TableC2PARaw::MINIMUM_SIZE,
    );
    assert!(result.is_err());
    let error = result.err().unwrap();
    assert!(matches!(
        error,
        FontIoError::LoadTableTruncated(FontTag::C2PA)
    ));
}

#[test]
fn test_table_c2pa_read_exact_with_invalid_manifest_store() {
    // There is enough data to read
    let mut data = vec![];
    data.extend_from_slice(&[0x00, 0x01]); // major_version
    data.extend_from_slice(&[0x00, 0x04]); // minor_version
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // active manifest uri offset
    data.extend_from_slice(&[0x00, 0x00]); // active manifest uri length
    data.extend_from_slice(&[0x00, 0x00]); // reserved
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x14]); // content_credential offset
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x04]); // content_credential
    data.extend_from_slice(b"ng"); // Incomplete content credential
                                   // length
    let mut reader = Cursor::new(data);
    // But when calling the method we don't tell it to read enough data to
    // capture the table
    let result = TableC2PA::from_reader_exact(&mut reader, 0, 24);
    assert!(result.is_err());
    let error = result.err().unwrap();
    assert!(matches!(
        error,
        FontIoError::LoadTableTruncated(FontTag::C2PA)
    ));
}

#[test]
fn test_table_c2pa_read_exact_with_invalid_uri_bytes() {
    // There is enough data to read
    let mut data = vec![];
    data.extend_from_slice(&[0x00, 0x01]); // major_version
    data.extend_from_slice(&[0x00, 0x04]); // minor_version
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x14]); // active manifest uri offset
    data.extend_from_slice(&[0x00, 0x04]); // active manifest uri length
    data.extend_from_slice(&[0x00, 0x00]); // reserved
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // content_credential offset
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // content_credential
                                                       // Then write 4 non-UTF-8 bytes
    data.extend_from_slice(&[0xff, 0xff, 0xff, 0xff]);
    // length
    let mut reader = Cursor::new(data);
    // But when calling the method we don't tell it to read enough data to
    // capture the table
    let result = TableC2PA::from_reader_exact(&mut reader, 0, 24);
    assert!(result.is_err());
    let error = result.err().unwrap();
    assert!(matches!(error, FontIoError::StringFromUtf8(_)));
}

// Used to generate a test file for the C2PA table in the format of a font table
#[ignore]
#[test]
fn test_table_c2pa_create() {
    let mut table = TableC2PA {
        active_manifest_uri: Some(
            "https://example.com/example/font/cc.c2pa".to_string(),
        ),
        ..Default::default()
    };
    // Make a fake manifest store that is 2048 bytes long
    let rng_gen = |seed: &mut u32| {
        const A: u32 = 48271;
        const M: u32 = 2147483647;
        *seed = seed.wrapping_mul(A).wrapping_add(M);
        (*seed >> 16) as u8
    };

    // Setup to do different bands so any compression will be
    // beneficial
    let bands = 14;
    // Setup for ~14KB of data
    let size = 1024 * bands;
    let mut manifest_store = vec![0; size];
    // Fill the manifest store with some random data
    for i in 0..bands {
        let mut seed: u32 = i as u32;
        let value = rng_gen(&mut seed);
        for j in 0..(size / bands) {
            // Generate some random data
            manifest_store[i * (size / bands) + j] = value;
        }
    }
    table.manifest_store = Some(manifest_store);
    let output_file = std::fs::File::create("faux_c2pa_table.bin").unwrap();
    let mut writer = std::io::BufWriter::new(output_file);
    let result = table.write(&mut writer);
    assert!(result.is_ok());
}
