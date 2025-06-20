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

//! DSIG SFNT table.

use std::{
    io::{Read, Seek, SeekFrom, Write},
    num::Wrapping,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    error::FontIoError, tag::FontTag, utils::u32_from_u16_pair,
    FontDataChecksum, FontDataExactRead, FontDataWrite, FontTable,
};

/// 'DSIG' font table, ignores actual signatures as we intend to only use this
/// as a stub DSIG table.
#[allow(non_snake_case)] // As named by Open Font Format / OpenType.
pub struct TableDSIG {
    /// Version of the DSIG table.
    pub version: u32,
    /// Number of signatures in the DSIG table.
    pub numSignatures: u16,
    /// Flags for the DSIG table.
    pub flags: u16,
    /// Data of the DSIG table.
    #[allow(dead_code)] // We don't actually use this data.
    pub data: Vec<u8>,
}

impl TableDSIG {
    /// The default version of the DSIG table.
    const DEFAULT_VERSION: u32 = 0x00000001;
    /// The flag to not resign the table.
    const DO_NOT_RESIGN: u16 = 0x0001;
    /// The size of a DSIG table.
    const MINIMUM_SIZE: usize = 8;

    /// Create an empty DSIG stub table.
    pub(crate) fn stub() -> Self {
        Self {
            version: Self::DEFAULT_VERSION,
            numSignatures: 0,
            flags: Self::DO_NOT_RESIGN,
            data: Vec::new(),
        }
    }

    /// Check if this DSIG table is a stub.
    pub(crate) fn is_stubbed(&self) -> bool {
        self.version == Self::DEFAULT_VERSION
            && self.numSignatures == 0
            && self.flags == Self::DO_NOT_RESIGN
            && self.data.is_empty()
    }
}

impl FontDataExactRead for TableDSIG {
    type Error = FontIoError;

    fn from_reader_exact<T: Read + Seek + ?Sized>(
        reader: &mut T,
        offset: u64,
        size: usize,
    ) -> Result<Self, Self::Error> {
        reader.seek(SeekFrom::Start(offset))?;
        if size < Self::MINIMUM_SIZE {
            Err(FontIoError::LoadTableTruncated(FontTag::DSIG))
        } else {
            let version = reader.read_u32::<BigEndian>()?;
            let num_signatures = reader.read_u16::<BigEndian>()?;
            let flags = reader.read_u16::<BigEndian>()?;
            let data_size = size - Self::MINIMUM_SIZE;
            let mut data = Vec::with_capacity(data_size);
            reader.take(data_size as u64).read_to_end(&mut data)?;
            Ok(TableDSIG {
                version,
                numSignatures: num_signatures,
                flags,
                data,
            })
        }
    }
}

impl FontDataWrite for TableDSIG {
    type Error = FontIoError;

    fn write<TDest: Write + ?Sized>(
        &self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        dest.write_u32::<BigEndian>(self.version)?;
        dest.write_u16::<BigEndian>(self.numSignatures)?;
        dest.write_u16::<BigEndian>(self.flags)?;

        Ok(())
    }
}

impl FontDataChecksum for TableDSIG {
    fn checksum(&self) -> std::num::Wrapping<u32> {
        let mut cksum = Wrapping(self.version);
        cksum += u32_from_u16_pair(self.numSignatures, self.flags);

        cksum
    }
}

impl FontTable for TableDSIG {
    fn len(&self) -> u32 {
        Self::MINIMUM_SIZE as u32
    }
}

#[cfg(test)]
#[path = "dsig_test.rs"]
mod tests;
