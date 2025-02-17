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
    println!("{:?}", error);
    assert!(matches!(error, FontIoError::NotEnoughBytes(_)));
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
    let result = TableC2PA::from_reader(&mut reader);
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
    let result = TableC2PA::from_reader(&mut reader);
    assert!(result.is_err());
    let error = result.err().unwrap();
    assert!(matches!(error, FontIoError::NotEnoughBytes(_)));
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
                                                       // create a cursor/reader around the data
    let mut reader = Cursor::new(data);
    let result = TableC2PA::from_reader(&mut reader);
    assert!(result.is_err());
    let error = result.err().unwrap();
    assert!(matches!(error, FontIoError::NotEnoughBytes(_)));
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
                                                       // create a cursor/reader around the data
    let mut reader = Cursor::new(data);
    let result = TableC2PA::from_reader(&mut reader);
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
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: Some("test".to_string()),
        manifest_store: Some(vec![1, 2, 3, 4]),
    };
    let checksum = table.checksum();
    let mut expected_checksum = Wrapping(0x00000000);
    // Make 32-bit big-endian checksum
    // Shift the major version by 16 bits to the left and add the minor version
    expected_checksum += Wrapping(65536 + 4);
    expected_checksum += Wrapping(20);
    // Shift the active manifest uri length by 16 bits to the left and add the
    // reserved field
    expected_checksum += Wrapping(4 * 65536);
    expected_checksum += Wrapping(24);
    expected_checksum += Wrapping(4);
    // Add the checksum of the uri
    expected_checksum += Wrapping(0x74657374);
    // Add the checksum of the content credential
    expected_checksum += Wrapping(0x01020304);

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
    let mut update_record = UpdateContentCredentialRecord::default();
    update_record.without_active_manifest_uri();
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
    let mut update_record = UpdateContentCredentialRecord::default();
    update_record.with_active_manifest_uri("new_test".to_string());
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
    let mut update_record = UpdateContentCredentialRecord::default();
    update_record.without_content_credentials();
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
    let mut update_record = UpdateContentCredentialRecord::default();
    update_record.with_content_credential(vec![5, 6, 7, 8]);
    _ = table.update_c2pa_record(update_record);
    assert_eq!(table.active_manifest_uri, Some("test".to_string()));
    assert_eq!(table.manifest_store, Some(vec![5, 6, 7, 8]));
}
