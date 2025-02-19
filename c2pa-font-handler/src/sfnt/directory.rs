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

//! SFNT font file directory and entries.

use std::{
    io::{Read, Seek, Write},
    mem::size_of,
    num::Wrapping,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    error::FontIoError, tag::FontTag, FontDataChecksum, FontDataExactRead,
    FontDataRead, FontDataWrite, FontDirectory, FontDirectoryEntry,
};

/// SFNT Table Directory Entry, from the OpenType spec.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed(1))] // As defined by the OpenType spec.
#[allow(non_snake_case)] // As defined by the OpenType spec.
pub struct SfntDirectoryEntry {
    pub(crate) tag: FontTag,
    pub(crate) checksum: u32,
    pub(crate) offset: u32,
    pub(crate) length: u32,
}

impl SfntDirectoryEntry {
    /// The size of an SFNT directory entry.
    pub const SIZE: usize = size_of::<Self>();
}

impl FontDataRead for SfntDirectoryEntry {
    type Error = FontIoError;

    fn from_reader<T: Read + Seek + ?Sized>(
        reader: &mut T,
    ) -> Result<Self, Self::Error> {
        let tag = FontTag::from_reader(reader)?;
        let checksum = reader.read_u32::<BigEndian>()?;
        let offset = reader.read_u32::<BigEndian>()?;
        let length = reader.read_u32::<BigEndian>()?;
        Ok(Self {
            tag,
            checksum,
            offset,
            length,
        })
    }
}

impl FontDataExactRead for SfntDirectoryEntry {
    type Error = FontIoError;

    fn from_reader_exact<T: Read + Seek + ?Sized>(
        reader: &mut T,
        offset: u64,
        size: usize,
    ) -> Result<Self, Self::Error> {
        if size != Self::SIZE {
            return Err(FontIoError::InvalidSizeForDirectoryEntry {
                expected: Self::SIZE,
                got: size,
            });
        }
        reader.seek(std::io::SeekFrom::Start(offset))?;
        Self::from_reader(reader)
    }
}

impl FontDataWrite for SfntDirectoryEntry {
    type Error = FontIoError;

    fn write<TDest: Write + ?Sized>(
        &self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        self.tag.write(dest)?;
        dest.write_u32::<BigEndian>(self.checksum)?;
        dest.write_u32::<BigEndian>(self.offset)?;
        dest.write_u32::<BigEndian>(self.length)?;
        Ok(())
    }
}

impl FontDataChecksum for SfntDirectoryEntry {
    fn checksum(&self) -> std::num::Wrapping<u32> {
        std::num::Wrapping(u32::from_be_bytes(self.tag.data()))
            + std::num::Wrapping(self.checksum)
            + std::num::Wrapping(self.offset)
            + std::num::Wrapping(self.length)
    }
}

impl FontDirectoryEntry for SfntDirectoryEntry {
    fn tag(&self) -> FontTag {
        self.tag
    }

    fn data_checksum(&self) -> u32 {
        self.checksum
    }

    fn offset(&self) -> u32 {
        self.offset
    }

    fn length(&self) -> u32 {
        self.length
    }
}

/// SFNT Directory is just an array of entries.
///
/// Undoubtedly there exists a more-oxidized way of just using Vec directly for
/// this... but maybe we don't want to? Note the choice of Vec over BTreeMap
/// here, which lets us keep non-compliant fonts as-is...
#[derive(Debug, Default)]
pub struct SfntDirectory {
    entries: Vec<SfntDirectoryEntry>,
}

impl SfntDirectory {
    /// Creates a new, empty `SfntDirectory`.
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Adds an entry to the directory.
    pub(crate) fn add_entry(&mut self, entry: SfntDirectoryEntry) {
        self.entries.push(entry);
    }

    /// Sorts the entries in the directory, based on the provided closure.
    pub(crate) fn sort_entries<F, K>(&mut self, f: F)
    where
        F: FnMut(&SfntDirectoryEntry) -> K,
        K: Ord,
    {
        self.entries.sort_by_key(f);
    }
}

impl FontDataExactRead for SfntDirectory {
    type Error = FontIoError;

    fn from_reader_exact<T: Read + Seek + ?Sized>(
        reader: &mut T,
        offset: u64,
        size: usize,
    ) -> Result<Self, Self::Error> {
        if size % SfntDirectoryEntry::SIZE != 0 {
            return Err(FontIoError::InvalidSizeForDirectory(size));
        }
        let entry_count = size / SfntDirectoryEntry::SIZE;
        reader.seek(std::io::SeekFrom::Start(offset))?;
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(SfntDirectoryEntry::from_reader(reader)?);
        }
        Ok(Self { entries })
    }
}

impl FontDataWrite for SfntDirectory {
    type Error = FontIoError;

    fn write<TDest: Write + ?Sized>(
        &self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        // Loop through our entries updating
        for entry in &self.entries {
            entry.write(dest)?;
        }
        Ok(())
    }
}

impl FontDataChecksum for SfntDirectory {
    fn checksum(&self) -> std::num::Wrapping<u32> {
        match self.entries.is_empty() {
            true => Wrapping(0_u32),
            false => self
                .entries
                .iter()
                .fold(Wrapping(0_u32), |cksum, entry| cksum + entry.checksum()),
        }
    }
}

impl FontDirectory for SfntDirectory {
    type Entry = SfntDirectoryEntry;

    fn from_reader_with_count<T: Read + Seek + ?Sized>(
        reader: &mut T,
        entry_count: usize,
    ) -> Result<Self, <Self as FontDataExactRead>::Error> {
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(SfntDirectoryEntry::from_reader(reader)?);
        }
        Ok(Self { entries })
    }

    fn entries(&self) -> &[Self::Entry] {
        &self.entries
    }

    fn physical_order(&self) -> Vec<&Self::Entry> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.offset);
        entries
    }
}

#[cfg(test)]
#[path = "directory_test.rs"]
mod tests;
