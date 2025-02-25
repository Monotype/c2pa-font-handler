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

//! woff1 font.

use std::{
    collections::BTreeMap,
    io::{Read, Seek},
};

use super::{
    directory::{Woff1Directory, Woff1DirectoryEntry},
    header::Woff1Header,
    Table,
};
use crate::{
    data::Data, error::FontIoError, tag::FontTag, utils::align_to_four, Font,
    FontDataExactRead, FontDataRead, FontDataWrite, FontDirectory, FontHeader,
    FontTable, MutFontDataWrite,
};

/// WOFF 1.0 WOFF header chunk name
const WOFF1_HEADER_CHUNK_NAME: FontTag = FontTag {
    data: *b"\x00\x00\x00w",
};

/// Pseudo-tag for the table directory
const WOFF1_DIRECTORY_CHUNK_NAME: FontTag = FontTag {
    data: *b"\x00\x00\x01D",
};

/// WOFF 1.0 / 2.0 XML metadata
const WOFF_METADATA_CHUNK_NAME: FontTag = FontTag {
    data: *b"\x7F\x7F\x7FM",
};

/// WOFF 1.0 / 2.0 trailing private data
const WOFF_PRIVATE_CHUNK_NAME: FontTag = FontTag {
    data: *b"\x7F\x7F\x7FP",
};
/// Implementation of an woff1 font.
#[derive(Default)]
pub struct Woff1Font {
    pub(crate) header: Woff1Header,
    pub(crate) directory: Woff1Directory,
    pub(crate) tables: BTreeMap<FontTag, Table>,
    pub(crate) meta: Option<Data>,
    pub(crate) private_data: Option<Data>,
}

impl FontDataRead for Woff1Font {
    type Error = FontIoError;

    fn from_reader<T: Read + Seek + ?Sized>(
        reader: &mut T,
    ) -> Result<Self, Self::Error> {
        let header = Woff1Header::from_reader(reader)?;
        let meta_length = header.metaLength;
        let private_length = header.privLength;
        let directory = Woff1Directory::from_reader_with_count(
            reader,
            header.num_tables() as usize,
        )?;
        let mut tables = BTreeMap::new();
        for entry in directory.entries() {
            let aligned_length = align_to_four(entry.compLength) as usize;
            let table = Data::from_reader_exact(
                reader,
                entry.offset as u64,
                aligned_length,
            )?;
            tables.insert(entry.tag, table);
        }
        let meta = if meta_length > 0 {
            let aligned_length = align_to_four(meta_length) as usize;
            Some(Data::from_reader_exact(
                reader,
                header.metaOffset as u64,
                aligned_length,
            )?)
        } else {
            None
        };
        let private_data = if private_length > 0 {
            Some(Data::from_reader_exact(
                reader,
                header.privOffset as u64,
                private_length as usize,
            )?)
        } else {
            None
        };
        Ok(Self {
            header,
            directory,
            tables,
            meta,
            private_data,
        })
    }
}

impl MutFontDataWrite for Woff1Font {
    type Error = FontIoError;

    fn write<TDest: std::io::Write + ?Sized>(
        &mut self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        let mut neo_header = Woff1Header::default();
        let mut neo_directory = Woff1Directory::default();

        neo_header.flavor = self.header.flavor;
        neo_header.length = self.header.length;
        neo_header.numTables = self.tables.len() as u16;
        neo_header.reserved = self.header.reserved;
        neo_header.totalSfntSize = self.header.totalSfntSize;
        neo_header.majorVersion = self.header.majorVersion;
        neo_header.minorVersion = self.header.minorVersion;
        neo_header.metaOffset = self.header.metaOffset;
        neo_header.metaLength = self.header.metaLength;
        neo_header.metaOrigLength = self.header.metaOrigLength;
        neo_header.privOffset = self.header.privOffset;
        neo_header.privLength = self.header.privLength;

        let new_table_count = self.tables.len() as u16;

        let mut running_offset = Woff1Header::SIZE as u32
            + new_table_count as u32 * Woff1DirectoryEntry::SIZE as u32;

        self.directory.physical_order().iter().for_each(|entry| {
            if self.tables.contains_key(&entry.tag) {
                let table = self.tables.get(&entry.tag).unwrap();
                let neo_entry = Woff1DirectoryEntry {
                    tag: entry.tag,
                    offset: running_offset,
                    compLength: entry.compLength,
                    origLength: entry.origLength,
                    origChecksum: entry.origChecksum,
                };
                neo_directory.add_entry(neo_entry);
                running_offset += align_to_four(table.len());
            }
        });

        neo_directory.sort_entries(|entry| entry.tag);
        if let Some(meta) = &self.meta {
            neo_header.metaOffset = running_offset;
            neo_header.metaLength = align_to_four(meta.len());
            running_offset += neo_header.metaLength;
        }
        if let Some(private) = &self.private_data {
            neo_header.privOffset = running_offset;
            neo_header.privLength = align_to_four(private.len());
        }
        self.header = neo_header;
        self.directory = neo_directory;
        self.header.write(dest)?;
        self.directory.write(dest)?;
        for entry in self.directory.physical_order().iter() {
            self.tables[&entry.tag].write(dest)?;
        }
        // If we have metadata, write it
        if let Some(meta) = &self.meta {
            meta.write(dest)?;
        }
        // If we have private data, write it
        if let Some(private_data) = &self.private_data {
            private_data.write(dest)?;
        }
        Ok(())
    }
}

impl Font for Woff1Font {
    type Directory = Woff1Directory;
    type Header = Woff1Header;
    type Table = Table;

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn directory(&self) -> &Self::Directory {
        &self.directory
    }

    fn contains_table(&self, tag: &FontTag) -> bool {
        self.tables.contains_key(tag)
    }

    fn table(&self, tag: &FontTag) -> Option<&Self::Table> {
        self.tables.get(tag)
    }
}

#[cfg(test)]
#[path = "font_test.rs"]
mod tests;
