// Copyright 2024 Monotype Imaging Inc.
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

//! SFNT font file header.

use std::{
    io::{Read, Seek, Write},
    num::Wrapping,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    error::FontIoError, magic::Magic, utils::u32_from_u16_pair,
    FontDataChecksum, FontDataExactRead, FontDataRead, FontDataWrite,
    FontHeader,
};

/// All the serialization structures so far have been defined using native
/// Rust types; should we go all-out in the other direction, and establish a
/// layer of "font" types (FWORD, FIXED, etc.)?
///
/// SFNT header, from the OpenType spec.
#[derive(Copy, Clone)]
#[repr(C, packed(1))] // As defined by the OpenType spec.
#[allow(non_snake_case)] // As defined by the OpenType spec.
pub struct SfntHeader {
    /// The SFNT version.
    pub sfntVersion: Magic,
    /// The number of tables in the font.
    pub numTables: u16,
    /// The search range.
    pub searchRange: u16,
    /// The entry selector.
    pub entrySelector: u16,
    /// The range shift.
    pub rangeShift: u16,
}

impl SfntHeader {
    /// The size of an SFNT header.
    pub(crate) const SIZE: usize = 12;
}

impl Default for SfntHeader {
    fn default() -> Self {
        Self {
            sfntVersion: Magic::TrueType,
            numTables: 0,
            searchRange: 0,
            entrySelector: 0,
            rangeShift: 0,
        }
    }
}
impl FontDataRead for SfntHeader {
    type Error = FontIoError;

    fn from_reader<T: Read + Seek + ?Sized>(
        reader: &mut T,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            sfntVersion: Magic::try_from(reader.read_u32::<BigEndian>()?)?,
            numTables: reader.read_u16::<BigEndian>()?,
            searchRange: reader.read_u16::<BigEndian>()?,
            entrySelector: reader.read_u16::<BigEndian>()?,
            rangeShift: reader.read_u16::<BigEndian>()?,
        })
    }
}

impl FontDataExactRead for SfntHeader {
    type Error = FontIoError;

    fn from_reader_exact<T: Read + Seek + ?Sized>(
        reader: &mut T,
        offset: u64,
        size: usize,
    ) -> Result<Self, Self::Error> {
        if size != Self::SIZE {
            return Err(FontIoError::InvalidSizeForHeader(size));
        }
        reader.seek(std::io::SeekFrom::Start(offset))?;
        Self::from_reader(reader)
    }
}

impl FontDataWrite for SfntHeader {
    type Error = FontIoError;

    fn write<TDest: Write + ?Sized>(
        &self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        dest.write_u32::<BigEndian>(self.sfntVersion as u32)?;
        dest.write_u16::<BigEndian>(self.numTables)?;
        dest.write_u16::<BigEndian>(self.searchRange)?;
        dest.write_u16::<BigEndian>(self.entrySelector)?;
        dest.write_u16::<BigEndian>(self.rangeShift)?;
        Ok(())
    }
}

impl FontDataChecksum for SfntHeader {
    fn checksum(&self) -> Wrapping<u32> {
        // 0x00
        Wrapping(self.sfntVersion as u32)
            // 0x04
            + u32_from_u16_pair(self.numTables, self.searchRange)
            // 0x08
            + u32_from_u16_pair(self.entrySelector, self.rangeShift)
    }
}

impl FontHeader for SfntHeader {
    fn num_tables(&self) -> u16 {
        self.numTables
    }
}

#[cfg(test)]
#[path = "header_test.rs"]
mod tests;
