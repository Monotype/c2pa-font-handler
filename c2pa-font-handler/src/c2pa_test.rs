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

//! Tests for C2PA

use super::*;

#[test]
fn test_record_getters() {
    let record = ContentCredentialRecord::default();
    assert_eq!(record.major_version(), 1);
    assert_eq!(record.minor_version(), 4);
    assert_eq!(record.active_manifest_uri(), None,);
    assert!(record.content_credential().is_none());
}

#[test]
fn test_record_builder() {
    let result = ContentCredentialRecord::builder()
        .with_version(1, 4)
        .with_active_manifest_uri("http://example.com/manifest".to_owned())
        .with_content_credential(vec![1, 2, 3, 4])
        .build();
    println!("{:?}", result);
    assert!(result.is_ok());
    let record = result.unwrap();
    if let ContentCredentialRecord {
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: Some(uri),
        content_credential: Some(credential),
    } = record
    {
        assert_eq!(uri, "http://example.com/manifest".to_string());
        assert_eq!(credential, vec![1u8, 2u8, 3u8, 4u8]);
    } else {
        panic!("Record does not match expected values");
    }
}

#[test]
fn test_record_builder_default() {
    let result = ContentCredentialRecord::builder().build();
    assert!(result.is_ok());
    let record = result.unwrap();
    if let ContentCredentialRecord {
        major_version: 1,
        minor_version: 4,
        active_manifest_uri: None,
        content_credential: None,
    } = record
    {
    } else {
        panic!("Record does not match expected values");
    }
}

#[test]
fn test_record_builder_invalid_major_version() {
    let result = ContentCredentialRecord::builder()
        .with_version(4, 4)
        .with_active_manifest_uri("http://example.com/manifest".to_owned())
        .with_content_credential(vec![1, 2, 3, 4])
        .build();
    assert!(result.is_err());
}

#[test]
fn test_record_builder_invalid_minor_version() {
    let result = ContentCredentialRecord::builder()
        .with_version(1, 5)
        .with_active_manifest_uri("http://example.com/manifest".to_owned())
        .with_content_credential(vec![1, 2, 3, 4])
        .build();
    assert!(result.is_err());
}

#[test]
fn test_update_record_removed_items() {
    let mut update_record = UpdateContentCredentialRecord {
        major_version: None,
        minor_version: None,
        active_manifest_uri: None,
        content_credential: None,
    };
    update_record.without_a_version();
    update_record.without_active_manifest_uri();
    update_record.without_content_credentials();
    assert!(matches!(
        update_record,
        UpdateContentCredentialRecord {
            major_version: Some(UpdateType::Remove),
            minor_version: Some(UpdateType::Remove),
            active_manifest_uri: Some(UpdateType::Remove),
            content_credential: Some(UpdateType::Remove),
        }
    ));
}

#[test]
fn test_update_record_updated_items() {
    let mut update_record = UpdateContentCredentialRecord {
        major_version: None,
        minor_version: None,
        active_manifest_uri: None,
        content_credential: None,
    };
    update_record.with_version(1, 4);
    update_record
        .with_active_manifest_uri("http://example.com/manifest".to_owned());
    update_record.with_content_credential(vec![1, 2, 3, 4]);
    assert!(matches!(
        update_record.major_version(),
        Some(UpdateType::Update(1))
    ));
    assert!(update_record.major_version().is_none());
    assert!(matches!(
        update_record.minor_version(),
        Some(UpdateType::Update(4))
    ));
    assert!(update_record.minor_version().is_none());
    assert!(matches!(
        update_record.take_active_manifest_uri(),
        Some(UpdateType::Update(uri)) if uri == "http://example.com/manifest"
    ));
    assert!(update_record.take_active_manifest_uri().is_none());
    assert!(matches!(
        update_record.take_content_credential(),
        Some(UpdateType::Update(credential)) if credential == vec![1, 2, 3, 4]
    ));
    assert!(update_record.take_content_credential().is_none());
}
