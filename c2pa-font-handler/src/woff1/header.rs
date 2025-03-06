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

//! WOFF1 font file header.

use std::{
    io::{Read, Seek, Write},
    mem::size_of,
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
/// WOFF1 header
#[derive(Copy, Clone, Debug)]
#[repr(C, packed(1))]
#[allow(non_snake_case)]
pub struct Woff1Header {
    /// The 'magic' number for WOFF1 files (i.e., 0x774F4646 as defined in <https://www.w3.org/TR/2012/REC-WOFF-20121213/>).
    pub(crate) signature: Magic,
    /// The "sfnt flavor" of the font.
    pub(crate) flavor: u32,
    /// The length of the WOFF file.
    pub(crate) length: u32,
    /// The number of tables in the font.
    pub(crate) numTables: u16,
    /// Reserved; must be 0.
    pub(crate) reserved: u16,
    /// Total size needed for the uncompressed font data, including the sfnt
    /// header, directory, and font tables (including padding).
    pub(crate) totalSfntSize: u32,
    /// Major version of the WOFF file format.
    pub(crate) majorVersion: u16,
    /// Minor version of the WOFF file format.
    pub(crate) minorVersion: u16,
    /// Offset to the 'metadata' block, from the beginning of the WOFF file.
    pub(crate) metaOffset: u32,
    /// Length of the 'metadata' block.
    pub(crate) metaLength: u32,
    /// Uncompressed size of the 'metadata' block.
    pub(crate) metaOrigLength: u32,
    /// Offset to the private data block, from the beginning of the WOFF file.
    pub(crate) privOffset: u32,
    /// Length of the private data block.
    pub(crate) privLength: u32,
}

impl Default for Woff1Header {
    fn default() -> Self {
        Self {
            signature: Magic::Woff,
            flavor: 0,
            length: 0,
            numTables: 0,
            reserved: 0,
            totalSfntSize: 0,
            majorVersion: 0,
            minorVersion: 0,
            metaOffset: 0,
            metaLength: 0,
            metaOrigLength: 0,
            privOffset: 0,
            privLength: 0,
        }
    }
}

impl Woff1Header {
    /// The size of an WOFF1 header.
    pub(crate) const SIZE: usize = size_of::<Self>();
}

impl FontDataRead for Woff1Header {
    type Error = FontIoError;

    fn from_reader<T: Read + Seek + ?Sized>(
        reader: &mut T,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            signature: Magic::try_from(reader.read_u32::<BigEndian>()?)?,
            flavor: reader.read_u32::<BigEndian>()?,
            length: reader.read_u32::<BigEndian>()?,
            numTables: reader.read_u16::<BigEndian>()?,
            reserved: reader.read_u16::<BigEndian>()?,
            totalSfntSize: reader.read_u32::<BigEndian>()?,
            majorVersion: reader.read_u16::<BigEndian>()?,
            minorVersion: reader.read_u16::<BigEndian>()?,
            metaOffset: reader.read_u32::<BigEndian>()?,
            metaLength: reader.read_u32::<BigEndian>()?,
            metaOrigLength: reader.read_u32::<BigEndian>()?,
            privOffset: reader.read_u32::<BigEndian>()?,
            privLength: reader.read_u32::<BigEndian>()?,
        })
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
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        dest.write_u32::<BigEndian>(self.signature as u32)?;
        dest.write_u32::<BigEndian>(self.flavor)?;
        dest.write_u32::<BigEndian>(self.length)?;
        dest.write_u16::<BigEndian>(self.numTables)?;
        dest.write_u16::<BigEndian>(self.reserved)?;
        dest.write_u32::<BigEndian>(self.totalSfntSize)?;
        dest.write_u16::<BigEndian>(self.majorVersion)?;
        dest.write_u16::<BigEndian>(self.minorVersion)?;
        dest.write_u32::<BigEndian>(self.metaOffset)?;
        dest.write_u32::<BigEndian>(self.metaLength)?;
        dest.write_u32::<BigEndian>(self.metaOrigLength)?;
        dest.write_u32::<BigEndian>(self.privOffset)?;
        dest.write_u32::<BigEndian>(self.privLength)?;
        Ok(())
    }
}

impl FontDataChecksum for Woff1Header {
    fn checksum(&self) -> Wrapping<u32> {
        Wrapping(self.signature as u32)
            + Wrapping(self.flavor)
            + Wrapping(self.length)
            + u32_from_u16_pair(self.numTables, self.reserved)
            + Wrapping(self.totalSfntSize)
            + u32_from_u16_pair(self.majorVersion, self.minorVersion)
            + Wrapping(self.metaOffset)
            + Wrapping(self.metaLength)
            + Wrapping(self.metaOrigLength)
            + Wrapping(self.privOffset)
            + Wrapping(self.privLength)
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
