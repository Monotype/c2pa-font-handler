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

//! 'head' SFNT table.

use std::{
    io::{Read, Seek, SeekFrom, Write},
    mem::size_of,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    error::FontIoError, tag::FontTag, utils::u32_from_u16_pair,
    FontDataChecksum, FontDataExactRead, FontDataWrite, FontTable,
};

/// Spec-mandated magic number for the 'head' table.
const HEAD_TABLE_MAGIC_NUMBER: u32 = 0x5f0f3cf5;
/// The 'head' table's checksumAdjustment value should be such that the
/// whole-font checksum comes out to this value.
pub(crate) const SFNT_EXPECTED_CHECKSUM: u32 = 0xb1b0afba;

/// 'head' font table
#[derive(Debug)]
#[repr(C, packed(1))]
#[allow(non_snake_case)] // As named by Open Font Format / OpenType.
pub struct TableHead {
    /// Major version number of the font.
    pub majorVersion: u16, // Note - Since we only modify checksumAdjustment,
    /// Minor version number of the font.
    pub minorVersion: u16, // we might just as well define this struct as
    /// Revision number of the font.
    pub fontRevision: u32, //    version_stuff: u8[8],
    /// Checksum adjustment.
    pub checksumAdjustment: u32, //    checksumAdjustment: u32,
    /// Magic number for the font.
    pub magicNumber: u32, //    rest_of_stuff: u8[42],
    /// Flags for the font.
    pub flags: u16,
    /// Units per em.
    pub unitsPerEm: u16,
    /// Date created.
    pub created: i64,
    /// Date modified.
    pub modified: i64,
    /// Minimum x.
    pub xMin: i16,
    /// Minimum y.
    pub yMin: i16,
    /// Maximum x.
    pub xMax: i16,
    /// Maximum y.
    pub yMax: i16,
    /// Mac style.
    pub macStyle: u16,
    /// Lowest PPEM.
    pub lowestRecPPEM: u16,
    /// Font direction hint.
    pub fontDirectionHint: i16,
    /// Index to loc format.
    pub indexToLocFormat: i16,
    /// Glyph data format.
    pub glyphDataFormat: i16,
}

impl TableHead {
    /// The size of a 'head' table.
    const SIZE: usize = size_of::<Self>();
}

impl FontDataExactRead for TableHead {
    type Error = FontIoError;

    fn from_reader_exact<T: Read + Seek + ?Sized>(
        reader: &mut T,
        offset: u64,
        size: usize,
    ) -> Result<Self, Self::Error> {
        reader.seek(SeekFrom::Start(offset))?;
        if size != Self::SIZE {
            Err(FontIoError::LoadTableTruncated(FontTag::HEAD))
        } else {
            let head = Self {
                // 0x00
                majorVersion: reader.read_u16::<BigEndian>()?,
                minorVersion: reader.read_u16::<BigEndian>()?,
                // 0x04
                fontRevision: reader.read_u32::<BigEndian>()?,
                // 0x08
                checksumAdjustment: reader.read_u32::<BigEndian>()?,
                // 0x0c
                magicNumber: reader.read_u32::<BigEndian>()?,
                // 0x10
                flags: reader.read_u16::<BigEndian>()?,
                unitsPerEm: reader.read_u16::<BigEndian>()?,
                // 0x14
                created: reader.read_i64::<BigEndian>()?,
                // 0x1c
                modified: reader.read_i64::<BigEndian>()?,
                // 0x24
                xMin: reader.read_i16::<BigEndian>()?,
                yMin: reader.read_i16::<BigEndian>()?,
                // 0x28
                xMax: reader.read_i16::<BigEndian>()?,
                yMax: reader.read_i16::<BigEndian>()?,
                // 0x2c
                macStyle: reader.read_u16::<BigEndian>()?,
                lowestRecPPEM: reader.read_u16::<BigEndian>()?,
                // 0x30
                fontDirectionHint: reader.read_i16::<BigEndian>()?,
                indexToLocFormat: reader.read_i16::<BigEndian>()?,
                // 0x34
                glyphDataFormat: reader.read_i16::<BigEndian>()?,
                // 0x36 - 54 bytes
                // TBD - Two bytes of padding to get to 56/0x38. Should we
                // seek/discard two more bytes, just to leave the stream in a
                // known state? Be nice if we didn't have to.
                //   1. On the one hand, whoever's invoking us could more-
                //      efficiently mess around with the offsets and padding.
                //   B. On the other, for the .write() code, we definitely push
                //      the "pad *yourself* up to four, impl!" approach
                //   III. Likewise the .checksum() code (although, because this
                //        is a simple checksum, the matter is moot; it doesn't
                //        matter whether we add '0_u16' to the total.
                //   IIII. (On clocks, IIII is a permissible Roman numeral) But
                //      what about that "simple" '.len()' call? Should it
                //      include the two pad bytes?
                // For now, the surrounding code doesn't care how the read
                // stream is left, so we don't do anything, since that is
                // simplest.
            };
            if head.magicNumber != HEAD_TABLE_MAGIC_NUMBER {
                return Err(FontIoError::InvalidHeadMagicNumber(
                    head.magicNumber,
                ));
            }
            Ok(head)
        }
    }
}

impl FontDataWrite for TableHead {
    type Error = FontIoError;

    fn write<TDest: Write + ?Sized>(
        &self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        // 0x00
        dest.write_u16::<BigEndian>(self.majorVersion)?;
        dest.write_u16::<BigEndian>(self.minorVersion)?;
        // 0x04
        dest.write_u32::<BigEndian>(self.fontRevision)?;
        // 0x08
        dest.write_u32::<BigEndian>(self.checksumAdjustment)?;
        dest.write_u32::<BigEndian>(self.magicNumber)?;
        // 0x10
        dest.write_u16::<BigEndian>(self.flags)?;
        dest.write_u16::<BigEndian>(self.unitsPerEm)?;
        // 0x14
        dest.write_i64::<BigEndian>(self.created)?;
        // 0x1c
        dest.write_i64::<BigEndian>(self.modified)?;
        // 0x24
        dest.write_i16::<BigEndian>(self.xMin)?;
        dest.write_i16::<BigEndian>(self.yMin)?;
        // 0x28
        dest.write_i16::<BigEndian>(self.xMax)?;
        dest.write_i16::<BigEndian>(self.yMax)?;
        // 0x2c
        dest.write_u16::<BigEndian>(self.macStyle)?;
        dest.write_u16::<BigEndian>(self.lowestRecPPEM)?;
        // 0x30
        dest.write_i16::<BigEndian>(self.fontDirectionHint)?;
        dest.write_i16::<BigEndian>(self.indexToLocFormat)?;
        // 0x34
        dest.write_i16::<BigEndian>(self.glyphDataFormat)?;
        // 0x36
        dest.write_u16::<BigEndian>(0_u16)?;
        // 0x38 - two bytes to get 54-byte 'head' up to nice round 56 bytes
        Ok(())
    }
}

impl FontDataChecksum for TableHead {
    fn checksum(&self) -> std::num::Wrapping<u32> {
        let mut cksum = u32_from_u16_pair(self.majorVersion, self.minorVersion);
        cksum += self.fontRevision;
        // Skip checksum adjustment (calculated as zero)
        cksum += self.magicNumber;
        cksum += u32_from_u16_pair(self.flags, self.unitsPerEm);

        let (low, high) = (self.created as u32, (self.created >> 32) as u32);
        cksum += low;
        cksum += high;

        let (low, high) = (self.modified as u32, (self.modified >> 32) as u32);
        cksum += low;
        cksum += high;

        cksum += u32_from_u16_pair(self.xMin as u16, self.yMin as u16);
        cksum += u32_from_u16_pair(self.xMax as u16, self.yMax as u16);
        cksum += u32_from_u16_pair(self.macStyle, self.lowestRecPPEM);
        cksum += u32_from_u16_pair(
            self.fontDirectionHint as u16,
            self.indexToLocFormat as u16,
        );
        cksum += u32_from_u16_pair(self.glyphDataFormat as u16, 0_u16);

        cksum
    }
}

impl FontTable for TableHead {
    fn data(&self) -> &[u8] {
        // SAFETY: The struct is packed, so the data is laid out in memory
        // exactly as it is in the file.
        unsafe {
            std::slice::from_raw_parts(
                self as *const _ as *const u8,
                Self::SIZE,
            )
        }
    }

    fn len(&self) -> u32 {
        Self::SIZE as u32
    }
}

#[cfg(test)]
#[path = "head_test.rs"]
mod tests;
