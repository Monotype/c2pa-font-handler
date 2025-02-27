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

//! Module for supporting adding, removing, and updating C2PA records.

use crate::{error::FontIoError, sfnt::table::TableC2PA};

/// Default major version
pub(crate) const DEFAULT_MAJOR_VERSION: u16 = 0u16;
/// Maximum major version
pub(crate) const MAX_MAJOR_VERSION: u16 = 0u16;
/// Default minor version
pub(crate) const DEFAULT_MINOR_VERSION: u16 = 1u16;

/// Support for adding/removing [`ContentCredentialRecord`] items.
pub trait C2PASupport {
    /// Error type returned when adding/removing C2PA records
    type Error;

    /// Adds the C2PA record
    fn add_c2pa_record(
        &mut self,
        record: ContentCredentialRecord,
    ) -> Result<(), Self::Error>;

    /// Indicates if a C2PA record is present
    fn has_c2pa(&self) -> bool;

    /// Gets the C2PA record
    fn get_c2pa(self) -> Result<Option<ContentCredentialRecord>, Self::Error>;

    /// Removes a C2PA record
    fn remove_c2pa_record(&mut self) -> Result<(), Self::Error>;
}

/// Support for updating content credential records.
pub trait UpdatableC2PA {
    /// Error type returned from updating C2PA records
    type Error;

    /// Updates a C2PA record, returning `None` if there wasn't a record before
    /// or the previous record if there was one.
    fn update_c2pa_record(
        &mut self,
        record: UpdateContentCredentialRecord,
    ) -> Result<(), Self::Error>;
}

/// A Content Credential record for supporting C2PA.
#[derive(Clone, Debug)]
pub struct ContentCredentialRecord {
    major_version: u16,
    minor_version: u16,
    active_manifest_uri: Option<String>,
    content_credential: Option<Vec<u8>>,
}

impl TryFrom<&TableC2PA> for ContentCredentialRecord {
    type Error = FontIoError;

    fn try_from(value: &TableC2PA) -> Result<Self, Self::Error> {
        let mut builder = ContentCredentialRecord::builder()
            .with_version(value.major_version, value.minor_version);
        if let Some(active_manifest_uri) = &value.active_manifest_uri {
            builder = builder
                .with_active_manifest_uri(active_manifest_uri.to_string());
        }
        if let Some(content_credential) = &value.manifest_store {
            builder =
                builder.with_content_credential(content_credential.to_vec());
        }
        builder.build()
    }
}

impl ContentCredentialRecord {
    /// Gets a builder to build a [`ContentCredentialRecord`] for use.
    pub fn builder() -> ContentCredentialRecordBuilder {
        ContentCredentialRecordBuilder::default()
    }

    /// Gets the major version of the record
    pub fn major_version(&self) -> u16 {
        self.major_version
    }

    /// Gets the minor version of the record
    pub fn minor_version(&self) -> u16 {
        self.minor_version
    }

    /// Gets the active manifest URI
    pub fn active_manifest_uri(&self) -> Option<&str> {
        self.active_manifest_uri.as_deref()
    }

    /// Gets the content credential
    pub fn content_credential(&self) -> Option<&[u8]> {
        self.content_credential.as_deref()
    }
}

impl Default for ContentCredentialRecord {
    fn default() -> Self {
        Self {
            major_version: DEFAULT_MAJOR_VERSION,
            minor_version: DEFAULT_MINOR_VERSION,
            active_manifest_uri: Default::default(),
            content_credential: Default::default(),
        }
    }
}

/// Update Type
#[derive(Debug)]
pub enum UpdateType<T: std::fmt::Debug> {
    /// Remove the value
    Remove,
    /// Update the value
    Update(T),
}
/// Update Content Credential Record
#[derive(Default, Debug)]
pub struct UpdateContentCredentialRecord {
    active_manifest_uri: Option<UpdateType<String>>,
    content_credential: Option<UpdateType<Vec<u8>>>,
}

impl UpdateContentCredentialRecord {
    /// Gets the active manifest URI
    pub fn take_active_manifest_uri(&mut self) -> Option<UpdateType<String>> {
        self.active_manifest_uri.take()
    }

    /// Gets the content credential
    pub fn take_content_credential(&mut self) -> Option<UpdateType<Vec<u8>>> {
        self.content_credential.take()
    }

    /// Gets a builder to build an [`UpdateContentCredentialRecord`] for use.
    pub fn builder() -> UpdateContentCredentialRecordBuilder {
        UpdateContentCredentialRecordBuilder::default()
    }
}

/// A Builder for an [`UpdateContentCredentialRecord`].
#[derive(Default)]
pub struct UpdateContentCredentialRecordBuilder {
    active_manifest_uri: Option<UpdateType<String>>,
    content_credential: Option<UpdateType<Vec<u8>>>,
}

impl UpdateContentCredentialRecordBuilder {
    /// Uses a specific active manifest URI
    pub fn with_active_manifest_uri(
        mut self,
        active_manifest_uri: String,
    ) -> Self {
        self.active_manifest_uri =
            Some(UpdateType::Update(active_manifest_uri));
        self
    }

    /// Removes the active manifest URI
    pub fn without_active_manifest_uri(mut self) -> Self {
        self.active_manifest_uri = Some(UpdateType::Remove);
        self
    }

    /// Uses the specified content credential
    pub fn with_content_credential(
        mut self,
        content_credential: Vec<u8>,
    ) -> Self {
        self.content_credential = Some(UpdateType::Update(content_credential));
        self
    }

    /// Removes the content credential
    pub fn without_content_credentials(mut self) -> Self {
        self.content_credential = Some(UpdateType::Remove);
        self
    }

    /// Builds the [`UpdateContentCredentialRecord`].
    pub fn build(self) -> UpdateContentCredentialRecord {
        UpdateContentCredentialRecord {
            active_manifest_uri: self.active_manifest_uri,
            content_credential: self.content_credential,
        }
    }
}

/// A Builder for a [`ContentCredentialRecord`].
#[derive(Default)]
pub struct ContentCredentialRecordBuilder {
    major_version: Option<u16>,
    minor_version: Option<u16>,
    active_manifest_uri: Option<String>,
    content_credential: Option<Vec<u8>>,
}

impl ContentCredentialRecordBuilder {
    /// Builds the [`ContentCredentialRecord`].
    pub fn build(
        self,
    ) -> Result<ContentCredentialRecord, crate::error::FontIoError> {
        // Grab the major version
        let major_version = self.major_version.unwrap_or(DEFAULT_MAJOR_VERSION);
        if major_version > MAX_MAJOR_VERSION {
            return Err(crate::error::FontIoError::InvalidC2paMajorVersion(
                major_version,
            ));
        }
        // And grab the minor version
        let minor_version = self.minor_version.unwrap_or(DEFAULT_MINOR_VERSION);
        // For now we only support 0.1
        if major_version == 0u16 && minor_version != 1u16 {
            return Err(crate::error::FontIoError::InvalidC2paMinorVersion(
                minor_version,
            ));
        }
        Ok(ContentCredentialRecord {
            major_version,
            minor_version,
            active_manifest_uri: self.active_manifest_uri,
            content_credential: self.content_credential,
        })
    }

    /// Uses a custom version for the record.
    pub fn with_version(
        mut self,
        major_version: u16,
        minor_version: u16,
    ) -> Self {
        self.major_version = Some(major_version);
        self.minor_version = Some(minor_version);
        self
    }

    /// Uses a specific active manifest URI
    pub fn with_active_manifest_uri(
        mut self,
        active_manifest_uri: String,
    ) -> Self {
        self.active_manifest_uri = Some(active_manifest_uri);
        self
    }

    /// Uses the specified content credential
    pub fn with_content_credential(
        mut self,
        content_credential: Vec<u8>,
    ) -> Self {
        self.content_credential = Some(content_credential);
        self
    }
}

#[cfg(test)]
#[path = "c2pa_test.rs"]
mod tests;
