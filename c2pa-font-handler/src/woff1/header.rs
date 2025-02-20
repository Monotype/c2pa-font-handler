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

//! WOFF1 font file header.

use std::{
    io::{Read, Seek, Write},
    mem::size_of,
    num::Wrapping,
};

use crate::{
    error::FontIoError, FontDataChecksum, FontDataExactRead, FontDataRead,
    FontDataWrite, FontHeader,
};

/// All the serialization structures so far have been defined using native
/// Rust types; should we go all-out in the other direction, and establish a
/// layer of "font" types (FWORD, FIXED, etc.)?
///
/// WOFF1 header
#[derive(Copy, Clone, Default)]
#[repr(C, packed(1))]
#[allow(non_snake_case)]
pub struct Woff1Header {
    /// The 'magic' number for WOFF1 files (i.e., 0x774F4646 as defined in https://www.w3.org/TR/2012/REC-WOFF-20121213/).
    pub signature: u32,
    /// The "sfnt flavor" of the font.
    pub flavor: u32,
    /// The length of the WOFF file.
    pub length: u32,
    /// The number of tables in the font.
    pub numTables: u16,
    /// Reserved; must be 0.
    pub reserved: u16,
    /// Total size needed for the uncompressed font data, including the sfnt
    /// header, directory, and font tables (including padding).
    pub totalSfntSize: u32,
    /// Major version of the WOFF file format.
    pub majorVersion: u16,
    /// Minor version of the WOFF file format.
    pub minorVersion: u16,
    /// Offset to the 'metadata' block, from the beginning of the WOFF file.
    pub metaOffset: u32,
    /// Length of the 'metadata' block.
    pub metaLength: u32,
    /// Uncompressed size of the 'metadata' block.
    pub metaOrigLength: u32,
    /// Offset to the private data block, from the beginning of the WOFF file.
    pub privOffset: u32,
    /// Length of the private data block.
    pub privLength: u32,
}

impl Woff1Header {
    /// The size of an WOFF1 header.
    pub(crate) const SIZE: usize = size_of::<Self>();
}

impl FontDataRead for Woff1Header {
    type Error = FontIoError;

    fn from_reader<T: Read + Seek + ?Sized>(
        _reader: &mut T,
    ) -> Result<Self, Self::Error> {
        todo!()
        /*
        Ok(Self {
            sfntVersion: Magic::try_from(reader.read_u32::<BigEndian>()?)?,
            numTables: reader.read_u16::<BigEndian>()?,
            searchRange: reader.read_u16::<BigEndian>()?,
            entrySelector: reader.read_u16::<BigEndian>()?,
            rangeShift: reader.read_u16::<BigEndian>()?,
        })
        */
    }
}

impl FontDataExactRead for Woff1Header {
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

impl FontDataWrite for Woff1Header {
    type Error = FontIoError;

    fn write<TDest: Write + ?Sized>(
        &self,
        _dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        todo!()
        /*
        dest.write_u32::<BigEndian>(self.sfntVersion as u32)?;
        dest.write_u16::<BigEndian>(self.numTables)?;
        dest.write_u16::<BigEndian>(self.searchRange)?;
        dest.write_u16::<BigEndian>(self.entrySelector)?;
        dest.write_u16::<BigEndian>(self.rangeShift)?;
        Ok(())
        */
    }
}

impl FontDataChecksum for Woff1Header {
    fn checksum(&self) -> Wrapping<u32> {
        todo!()
        /*
        // 0x00
        Wrapping(self.sfntVersion as u32)
            // 0x04
            + u32_from_u16_pair(self.numTables, self.searchRange)
            // 0x08
            + u32_from_u16_pair(self.entrySelector, self.rangeShift)
            */
    }
}

impl FontHeader for Woff1Header {
    fn num_tables(&self) -> u16 {
        self.numTables
    }
}

#[cfg(test)]
#[path = "header_test.rs"]
mod tests;
