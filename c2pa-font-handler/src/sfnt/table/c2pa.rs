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

//! C2PA table.

use std::{
    io::{Read, Seek, SeekFrom},
    mem::size_of,
    num::Wrapping,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    c2pa::{
        UpdatableC2PA, UpdateType, DEFAULT_MAJOR_VERSION, DEFAULT_MINOR_VERSION,
    },
    error::FontIoError,
    tag::FontTag,
    utils::{self, u32_from_u16_pair},
    FontDataChecksum, FontDataExactRead, FontDataRead, FontDataWrite,
    FontTable,
};

/// 'C2PA' font table
#[repr(C, packed(1))]
#[allow(non_snake_case)] // As named by Open Font Format / OpenType.
pub struct TableC2PARaw {
    /// Specifies the major version of the C2PA font table.
    pub majorVersion: u16,
    /// Specifies the minor version of the C2PA font table.
    pub minorVersion: u16,
    /// Offset from the beginning of the C2PA font table to the section
    /// containing a URI to the active manifest. If a URI is not provided a
    /// NULL offset = 0x0000 should be used.
    pub activeManifestUriOffset: u32,
    /// Length of URI in bytes.
    pub activeManifestUriLength: u16,
    /// Reserved for future use.
    pub reserved: u16,
    /// Offset from the beginning of the C2PA font table to the section
    /// containing a C2PA Manifest Store. If a Manifest Store is not provided a
    /// NULL offset = 0x0000 should be used.
    pub manifestStoreOffset: u32,
    /// Length of the C2PA Manifest Store data in bytes.
    pub manifestStoreLength: u32,
}

impl TableC2PARaw {
    /// The minimum required size of a C2PA table.
    const MINIMUM_SIZE: usize = 20;

    pub(crate) fn from_table(c2pa: &TableC2PA) -> Result<Self, FontIoError> {
        Ok(Self {
            majorVersion: c2pa.major_version,
            minorVersion: c2pa.minor_version,
            activeManifestUriOffset: if let Some(_uri) =
                &c2pa.active_manifest_uri
            {
                Self::MINIMUM_SIZE as u32
            } else {
                0
            },
            activeManifestUriLength: if let Some(uri) =
                &c2pa.active_manifest_uri
            {
                uri.len() as u16
            } else {
                0
            },
            reserved: 0,
            manifestStoreOffset: if let Some(_manifest_store) =
                &c2pa.manifest_store
            {
                size_of::<TableC2PARaw>() as u32
                    + if let Some(uri) = &c2pa.active_manifest_uri {
                        uri.len() as u32
                    } else {
                        0_u32
                    }
            } else {
                0_u32
            },
            manifestStoreLength: if let Some(store) = &c2pa.manifest_store {
                store.len() as u32
            } else {
                0
            },
        })
    }
}

impl FontDataRead for TableC2PARaw {
    type Error = FontIoError;

    fn from_reader<T: Read + Seek + ?Sized>(
        reader: &mut T,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            majorVersion: reader
                .read_u16::<BigEndian>()
                .map_err(FontIoError::NotEnoughBytes)?,
            minorVersion: reader
                .read_u16::<BigEndian>()
                .map_err(FontIoError::NotEnoughBytes)?,
            activeManifestUriOffset: reader
                .read_u32::<BigEndian>()
                .map_err(FontIoError::NotEnoughBytes)?,
            activeManifestUriLength: reader
                .read_u16::<BigEndian>()
                .map_err(FontIoError::NotEnoughBytes)?,
            reserved: reader
                .read_u16::<BigEndian>()
                .map_err(FontIoError::NotEnoughBytes)?,
            manifestStoreOffset: reader
                .read_u32::<BigEndian>()
                .map_err(FontIoError::NotEnoughBytes)?,
            manifestStoreLength: reader
                .read_u32::<BigEndian>()
                .map_err(FontIoError::NotEnoughBytes)?,
        })
    }
}

impl FontDataWrite for TableC2PARaw {
    type Error = FontIoError;

    fn write<TDest: std::io::Write + ?Sized>(
        &self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        dest.write_u16::<BigEndian>(self.majorVersion)?;
        dest.write_u16::<BigEndian>(self.minorVersion)?;
        dest.write_u32::<BigEndian>(self.activeManifestUriOffset)?;
        dest.write_u16::<BigEndian>(self.activeManifestUriLength)?;
        dest.write_u16::<BigEndian>(self.reserved)?;
        dest.write_u32::<BigEndian>(self.manifestStoreOffset)?;
        dest.write_u32::<BigEndian>(self.manifestStoreLength)?;
        Ok(())
    }
}

impl FontDataChecksum for TableC2PARaw {
    fn checksum(&self) -> Wrapping<u32> {
        let mut cksum = u32_from_u16_pair(self.majorVersion, self.minorVersion);
        cksum += self.activeManifestUriOffset;
        cksum += u32_from_u16_pair(self.activeManifestUriLength, self.reserved);
        cksum += self.manifestStoreOffset;
        cksum += self.manifestStoreLength;
        cksum
    }
}

impl FontTable for TableC2PARaw {
    fn len(&self) -> u32 {
        Self::MINIMUM_SIZE as u32
    }
}

/// 'C2PA' font table, fully loaded.
#[derive(Clone, Debug)]
pub struct TableC2PA {
    /// Specifies the major version of the C2PA font table.
    pub major_version: u16,
    /// Specifies the minor version of the C2PA font table.
    pub minor_version: u16,
    /// Optional URI to an active manifest
    pub active_manifest_uri: Option<String>,
    /// Optional embedded manifest store
    pub manifest_store: Option<Vec<u8>>,
}

impl FontDataRead for TableC2PA {
    type Error = FontIoError;

    fn from_reader<T: Read + Seek + ?Sized>(
        reader: &mut T,
    ) -> Result<Self, Self::Error> {
        // Grab the current position as the position of the C2PA table
        // from the start, as we will need it later to get the active
        // manifest uri and the manifest store.
        let table_position = reader.stream_position()?;
        // Placeholders
        let mut active_manifest_uri: Option<String> = None;
        let mut manifest_store: Option<Vec<u8>> = None;

        // Read in the raw C2PA table from the reader
        let raw_table: TableC2PARaw = TableC2PARaw::from_reader(reader)?;

        // If the active manifest URI offset is greater than 0, then we will
        // read it in
        if raw_table.activeManifestUriOffset > 0 {
            let mut uri_bytes: Vec<u8> =
                vec![0; raw_table.activeManifestUriLength as usize];
            reader.seek(SeekFrom::Start(
                table_position + raw_table.activeManifestUriOffset as u64,
            ))?;
            reader
                .read_exact(&mut uri_bytes)
                .map_err(FontIoError::NotEnoughBytes)?;
            active_manifest_uri = Some(
                String::from_utf8(uri_bytes)
                    .map_err(FontIoError::StringFromUtf8)?,
            );
        }
        // If the manifest store offset is greater than 0, then we will
        // read it in
        if raw_table.manifestStoreOffset > 0 {
            let mut store_bytes: Vec<u8> =
                vec![0; raw_table.manifestStoreLength as usize];
            reader.seek(SeekFrom::Start(
                table_position + raw_table.manifestStoreOffset as u64,
            ))?;
            reader
                .read_exact(&mut store_bytes)
                .map_err(FontIoError::NotEnoughBytes)?;
            manifest_store = Some(store_bytes);
        }

        // Return the expanded table
        Ok(Self {
            major_version: raw_table.majorVersion,
            minor_version: raw_table.minorVersion,
            active_manifest_uri,
            manifest_store,
        })
    }
}

impl FontDataExactRead for TableC2PA {
    type Error = FontIoError;

    fn from_reader_exact<T: Read + Seek + ?Sized>(
        reader: &mut T,
        offset: u64,
        size: usize,
    ) -> Result<Self, Self::Error> {
        if size < size_of::<TableC2PARaw>() {
            return Err(FontIoError::LoadTableTruncated(FontTag::C2PA));
        }
        reader.seek(SeekFrom::Start(offset))?;
        // TODO: This is a bit of a hack, but it works for now.
        //       We need to make sure the amount read is exactly the size of
        //       `size`.
        Self::from_reader(reader)
    }
}

impl FontDataWrite for TableC2PA {
    type Error = FontIoError;

    fn write<TDest: std::io::Write + ?Sized>(
        &self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        let raw_table = TableC2PARaw::from_table(self)?;
        raw_table.write(dest)?;
        if let Some(uri) = &self.active_manifest_uri {
            dest.write_all(uri.as_bytes())?;
        }
        if let Some(store) = &self.manifest_store {
            dest.write_all(store)?;
        }
        Ok(())
    }
}

impl FontDataChecksum for TableC2PA {
    fn checksum(&self) -> Wrapping<u32> {
        let raw_table = TableC2PARaw::from_table(self).unwrap();
        let header_cksum = raw_table.checksum();
        let uri_cksum = if let Some(uri) = &self.active_manifest_uri {
            utils::checksum(uri.as_bytes())
        } else {
            Wrapping(0)
        };
        let store_cksum = if let Some(store) = &self.manifest_store {
            utils::checksum(store)
        } else {
            Wrapping(0)
        };
        header_cksum + uri_cksum + store_cksum
    }
}

impl FontTable for TableC2PA {
    fn len(&self) -> u32 {
        // The length of the table is the length of the raw table plus the
        // length of the active manifest URI and the manifest store.
        let mut len = size_of::<TableC2PARaw>() as u32;
        if let Some(uri) = &self.active_manifest_uri {
            len += uri.len() as u32;
        }
        if let Some(store) = &self.manifest_store {
            len += store.len() as u32;
        }
        len
    }
}

impl Default for TableC2PA {
    fn default() -> Self {
        // TODO: Who should own the default major/minor versions???
        Self {
            major_version: DEFAULT_MAJOR_VERSION,
            minor_version: DEFAULT_MINOR_VERSION,
            active_manifest_uri: Default::default(),
            manifest_store: Default::default(),
        }
    }
}

impl UpdatableC2PA for TableC2PA {
    type Error = FontIoError;

    fn update_c2pa_record(
        &mut self,
        record: crate::c2pa::UpdateContentCredentialRecord,
    ) -> Result<(), Self::Error> {
        let mut record = record;
        match record.take_active_manifest_uri() {
            Some(UpdateType::Remove) => {
                self.active_manifest_uri = None;
            }
            Some(UpdateType::Update(uri)) => {
                self.active_manifest_uri = Some(uri.to_string());
            }
            None => {}
        };
        match record.take_content_credential() {
            Some(UpdateType::Remove) => {
                self.manifest_store = None;
            }
            Some(UpdateType::Update(store)) => {
                self.manifest_store = Some(store.to_vec());
            }
            None => {}
        };
        Ok(())
    }
}

#[cfg(test)]
#[path = "c2pa_test.rs"]
mod tests;
