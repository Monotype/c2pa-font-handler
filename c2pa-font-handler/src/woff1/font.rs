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

//! woff1 font.

use std::{
    collections::BTreeMap,
    io::{Read, Seek},
};

use super::{
    directory::Woff1Directory, header::Woff1Header, table::NamedTable,
};
use crate::{
    error::FontIoError, tag::FontTag, Font, FontDataExactRead, FontDataRead,
    FontDirectory, FontHeader, MutFontDataWrite,
};

/// Pseudo-tag for the table directory
const _WOFF1_DIRECTORY_CHUNK_NAME: FontTag = FontTag { data: *b" DIR" };

/// Implementation of an woff1 font.
#[derive(Default)]
pub struct Woff1Font {
    header: Woff1Header,
    directory: Woff1Directory,
    tables: BTreeMap<FontTag, NamedTable>,
}

impl FontDataRead for Woff1Font {
    type Error = FontIoError;

    fn from_reader<T: Read + Seek + ?Sized>(
        reader: &mut T,
    ) -> Result<Self, Self::Error> {
        let header = Woff1Header::from_reader(reader)?;
        let directory = Woff1Directory::from_reader_with_count(
            reader,
            header.num_tables() as usize,
        )?;
        let mut tables = BTreeMap::new();
        for entry in directory.entries() {
            let table = NamedTable::from_reader_exact(
                &entry.tag,
                reader,
                entry.offset as u64,
                entry.compLength as usize,
            )?;
            tables.insert(entry.tag, table);
        }
        Ok(Self {
            header,
            directory,
            tables,
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
        _dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}

impl Font for Woff1Font {
    type Directory = Woff1Directory;
    type Header = Woff1Header;
    type Table = NamedTable;

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
