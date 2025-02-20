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

use super::{directory::Woff1Directory, header::Woff1Header, table::Table};
use crate::{
    error::FontIoError, tag::FontTag, Font, FontDataExactRead, FontDataRead,
    FontDataWrite, FontDirectory, FontHeader, MutFontDataWrite,
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
    header: Woff1Header,
    directory: Woff1Directory,
    tables: BTreeMap<FontTag, Table>,
    meta: Option<Table>,
    private_data: Option<Table>,
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
            let table = Table::from_reader_exact(
                reader,
                entry.offset as u64,
                entry.compLength as usize,
            )?;
            tables.insert(entry.tag, table);
        }
        let meta = if meta_length > 0 {
            Some(Table::from_reader_exact(
                reader,
                header.metaOffset as u64,
                meta_length as usize,
            )?)
        } else {
            None
        };
        let private_data = if private_length > 0 {
            Some(Table::from_reader_exact(
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

impl FontDataExactRead for Woff1Font {
    type Error = FontIoError;

    fn from_reader_exact<T: Read + Seek + ?Sized>(
        reader: &mut T,
        offset: u64,
        _size: usize,
    ) -> Result<Self, Self::Error> {
        reader.seek(std::io::SeekFrom::Start(offset))?;
        Self::from_reader(reader)
    }
}

impl MutFontDataWrite for Woff1Font {
    type Error = FontIoError;

    fn write<TDest: std::io::Write + ?Sized>(
        &mut self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        // Write the header
        self.header.write(dest)?;
        // Write the directory
        self.directory.write(dest)?;
        // Write out all of the table entries
        for entry in self.directory.entries() {
            let table = self.tables.get(&entry.tag).unwrap();
            table.write(dest)?;
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
