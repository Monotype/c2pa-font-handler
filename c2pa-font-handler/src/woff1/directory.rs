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

//! WOFF1 font file directory and entries.

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

/// WOFF1 Table Directory Entry, from the OpenType spec.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed(1))] // As defined by the OpenType spec.
#[allow(non_snake_case)] // As defined by the OpenType spec.
pub struct Woff1DirectoryEntry {
    pub(crate) tag: FontTag,
    pub(crate) offset: u32,
    pub(crate) compLength: u32,
    pub(crate) origLength: u32,
    pub(crate) origChecksum: u32,
}

impl Woff1DirectoryEntry {
    /// The size of an WOFF1 directory entry.
    pub const SIZE: usize = size_of::<Self>();
}

impl FontDataRead for Woff1DirectoryEntry {
    type Error = FontIoError;

    fn from_reader<T: Read + Seek + ?Sized>(
        reader: &mut T,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            tag: FontTag::from_reader(reader)?,
            offset: reader.read_u32::<BigEndian>()?,
            compLength: reader.read_u32::<BigEndian>()?,
            origLength: reader.read_u32::<BigEndian>()?,
            origChecksum: reader.read_u32::<BigEndian>()?,
        })
    }
}

impl FontDataExactRead for Woff1DirectoryEntry {
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

impl FontDataWrite for Woff1DirectoryEntry {
    type Error = FontIoError;

    fn write<TDest: Write + ?Sized>(
        &self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        self.tag.write(dest)?;
        dest.write_u32::<BigEndian>(self.offset)?;
        dest.write_u32::<BigEndian>(self.compLength)?;
        dest.write_u32::<BigEndian>(self.origLength)?;
        dest.write_u32::<BigEndian>(self.origChecksum)?;
        Ok(())
    }
}

impl FontDataChecksum for Woff1DirectoryEntry {
    fn checksum(&self) -> Wrapping<u32> {
        Wrapping(u32::from_be_bytes(self.tag.data()))
            + Wrapping(self.offset)
            + Wrapping(self.compLength)
            + Wrapping(self.origLength)
            + Wrapping(self.origChecksum)
    }
}

impl FontDirectoryEntry for Woff1DirectoryEntry {
    fn tag(&self) -> FontTag {
        self.tag
    }

    fn data_checksum(&self) -> u32 {
        self.origChecksum
    }

    fn offset(&self) -> u32 {
        self.offset
    }

    fn length(&self) -> u32 {
        self.compLength
    }
}

/// WOFF1 Directory is just an array of entries.
///
/// Undoubtedly there exists a more-oxidized way of just using Vec directly for
/// this... but maybe we don't want to? Note the choice of Vec over BTreeMap
/// here, which lets us keep non-compliant fonts as-is...
#[derive(Debug, Default)]
pub struct Woff1Directory {
    entries: Vec<Woff1DirectoryEntry>,
}

#[allow(dead_code)] // TODO: Remove once implemented.
impl Woff1Directory {
    /// Creates a new, empty `Woff1Directory`.
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Adds an entry to the directory.
    pub(crate) fn add_entry(&mut self, entry: Woff1DirectoryEntry) {
        self.entries.push(entry);
    }

    /// Sorts the entries in the directory, based on the provided closure.
    pub(crate) fn sort_entries<F, K>(&mut self, f: F)
    where
        F: FnMut(&Woff1DirectoryEntry) -> K,
        K: Ord,
    {
        self.entries.sort_by_key(f);
    }
}

impl FontDataExactRead for Woff1Directory {
    type Error = FontIoError;

    fn from_reader_exact<T: Read + Seek + ?Sized>(
        reader: &mut T,
        offset: u64,
        size: usize,
    ) -> Result<Self, Self::Error> {
        if size % Woff1DirectoryEntry::SIZE != 0 {
            return Err(FontIoError::InvalidSizeForDirectory(size));
        }
        let entry_count = size / Woff1DirectoryEntry::SIZE;
        reader.seek(std::io::SeekFrom::Start(offset))?;
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(Woff1DirectoryEntry::from_reader(reader)?);
        }
        Ok(Self { entries })
    }
}

impl FontDataWrite for Woff1Directory {
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

impl FontDataChecksum for Woff1Directory {
    fn checksum(&self) -> Wrapping<u32> {
        match self.entries.is_empty() {
            true => Wrapping(0_u32),
            false => self
                .entries
                .iter()
                .fold(Wrapping(0_u32), |cksum, entry| cksum + entry.checksum()),
        }
    }
}

impl FontDirectory for Woff1Directory {
    type Entry = Woff1DirectoryEntry;

    fn from_reader_with_count<T: Read + Seek + ?Sized>(
        reader: &mut T,
        entry_count: usize,
    ) -> Result<Self, <Self as FontDataExactRead>::Error> {
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(Woff1DirectoryEntry::from_reader(reader)?);
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
