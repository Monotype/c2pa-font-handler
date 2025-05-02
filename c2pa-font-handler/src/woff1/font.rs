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
    collections::{btree_map::Entry, BTreeMap},
    fmt::Display,
    io::{Read, Seek, SeekFrom},
};

use super::{
    directory::{Woff1Directory, Woff1DirectoryEntry},
    header::Woff1Header,
    table::NamedTable,
};
use crate::{
    c2pa::C2PASupport,
    chunks::{ChunkPosition, ChunkReader, ChunkTypeTrait},
    data::Data,
    error::FontIoError,
    sfnt::table::TableC2PA,
    tag::FontTag,
    utils::align_to_four,
    Font, FontDataChecksum, FontDataExactRead, FontDataRead, FontDataWrite,
    FontDirectory, FontDirectoryEntry, FontHeader, FontTable, MutFontDataWrite,
};

/// Implementation of an woff1 font.
pub struct Woff1Font {
    pub(crate) header: Woff1Header,
    pub(crate) directory: Woff1Directory,
    pub(crate) tables: BTreeMap<FontTag, NamedTable>,
    pub(crate) metadata: Option<Data>,
    pub(crate) private_data: Option<Data>,
}

impl FontDataRead for Woff1Font {
    type Error = FontIoError;

    fn from_reader<T: Read + Seek + ?Sized>(
        reader: &mut T,
    ) -> Result<Self, Self::Error> {
        // Read in the WOFF1 header
        let header = Woff1Header::from_reader(reader)?;
        // Determine if we have extension metadata to read
        let meta_length = header.metaLength;
        // Determine if we have private data to read
        let private_length = header.privLength;
        // Read in the directory
        let directory = Woff1Directory::from_reader_with_count(
            reader,
            header.num_tables() as usize,
        )?;
        // And setup to read the contents of the tables
        let mut tables = BTreeMap::new();

        // TODO: Create a new decompression/compression type/stream that takes
        // the reader this will be responsible for decompressing the
        // data as it is read in

        for entry in directory.entries() {
            // check if the entry is compressed
            let table = if entry.compLength < entry.origLength {
                // If it is compressed, we need to decompress it
                reader.seek(SeekFrom::Start(entry.offset as u64))?;
                let mut compressed_data = vec![0; entry.origLength as usize];
                let mut decompress_reader =
                    crate::compression::DecompressingReader::new(reader);
                // Decompress the data
                decompress_reader.read_exact(&mut compressed_data)?;
                let mut decompressed_cursor =
                    std::io::Cursor::new(compressed_data);
                let table = NamedTable::from_reader_exact(
                    &entry.tag(),
                    &mut decompressed_cursor,
                    0,
                    entry.origLength as usize,
                )?;
                table
            } else {
                // Read in the table data
                let table = NamedTable::from_reader_exact(
                    &entry.tag(),
                    reader,
                    entry.offset as u64,
                    entry.length() as usize,
                )?;
                table
            };
            tables.insert(entry.tag, table);
        }
        // If we had extension metadata to read, read it
        let meta = if meta_length > 0 {
            Some(Data::from_reader_exact(
                reader,
                header.metaOffset as u64,
                meta_length as usize,
            )?)
        } else {
            None
        };
        // If we had private data to read, read it
        let private_data = if private_length > 0 {
            Some(Data::from_reader_exact(
                reader,
                header.privOffset as u64,
                private_length as usize,
            )?)
        } else {
            None
        };

        // Return the WOFF1 font
        Ok(Self {
            header,
            directory,
            tables,
            metadata: meta,
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
        // Setup to write our new header and directory
        let mut neo_header = Woff1Header::default();
        let mut neo_directory = Woff1Directory::default();

        // Fill in the new header with the old header's values
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

        // Fill in the new directory with the old directory's values
        let new_table_count = self.tables.len() as u16;

        // Use a running offset to calculate the new offsets
        let mut running_offset = Woff1Header::SIZE as u32
            + new_table_count as u32 * Woff1DirectoryEntry::SIZE as u32;

        // Iterate over the old directory and add entries to the new directory
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

        // TODO: Create a compressor to check if compressing the C2PA table
        // is worth it. Currently, we are not compressing it, but we may want to
        // in the future. For now, we will just add it to the directory as is.

        // TODO: Need a wrapper stream of a stream, since Zlib takes ownership
        // of the stream and we need to pass it to the compressor. This
        // will be a work in progress.

        if let Some(c2pa) = self.tables.get(&FontTag::C2PA) {
            if !self
                .directory
                .entries()
                .iter()
                .any(|entry| entry.tag == FontTag::C2PA)
            {
                let neo_entry = Woff1DirectoryEntry {
                    tag: FontTag::C2PA,
                    offset: running_offset,
                    compLength: c2pa.len(),
                    origLength: c2pa.len(),
                    origChecksum: c2pa.checksum().0,
                };
                neo_directory.add_entry(neo_entry);
                running_offset += align_to_four(c2pa.len());
            }
        }
        // Sort the new directory by tag
        neo_directory.sort_entries(|entry| entry.tag);

        // If we have extension metadata, update the header
        if let Some(meta) = &self.metadata {
            neo_header.metaOffset = running_offset;
            let aligned_length = align_to_four(meta.len());
            neo_header.metaLength = meta.len();
            running_offset += aligned_length;
        }

        // If we have private data, update the header
        if let Some(private) = &self.private_data {
            neo_header.privOffset = running_offset;
            neo_header.privLength = private.len();
        }
        // Update ourselves with the new header and directory
        self.header = neo_header;
        self.directory = neo_directory;

        // Write the header and directory
        self.header.write(dest)?;
        self.directory.write(dest)?;
        // And write out the tables
        for entry in self.directory.physical_order().iter() {
            self.tables[&entry.tag].write(dest)?;
        }
        // If we have metadata, write it
        if let Some(meta) = &self.metadata {
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

/// Pseudo tag for WOFF header chunk
const WOFF_HEADER_CHUNK_NAME: FontTag = FontTag {
    data: *b"\x00\x00\x00W",
};
/// Pseudo tag for WOFF directory chunk
const WOFF_DIRECTORY_CHUNK_NAME: FontTag = FontTag {
    data: *b"\x00\x00\x01D",
};
/// Pseudo tag for WOFF metadata chunk
const WOFF_METADATA_CHUNK_NAME: FontTag = FontTag {
    data: *b"\x7F\x7F\x7Fm",
};
/// Pseudo tag for WOFF private data chunk
const WOFF_PRIVATE_DATA_CHUNK_NAME: FontTag = FontTag {
    data: *b"\x7F\x7F\x7FP",
};

/// WOFF chunk type
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WoffChunkType {
    /// Header
    Header,
    /// Directory entry
    DirectoryEntry,
    /// Table data
    TableData,
    /// Metadata
    Metadata,
    /// Private data
    ///
    /// # Remarks
    /// Currently, the thinking is to put the C2PA data in the private data,
    /// but this may change.
    Private,
}

impl Display for WoffChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WoffChunkType::Header => write!(f, "Header"),
            WoffChunkType::DirectoryEntry => write!(f, "Directory Entry"),
            WoffChunkType::TableData => write!(f, "Table Data"),
            WoffChunkType::Metadata => write!(f, "Metadata"),
            WoffChunkType::Private => write!(f, "Private Data"),
        }
    }
}

impl ChunkTypeTrait for WoffChunkType {
    /// Currently this assumes the private data of a WOFF font will be excluded,
    /// but it is still a work in progress
    fn should_hash(&self) -> bool {
        match self {
            // At the moment, the private part is the only section excluded from
            // hashing
            WoffChunkType::Private => false,
            _ => true,
        }
    }
}

// NOTE: This is still a work in progress, as support C2PA in WOFF has not be
// fleshed out yet
impl ChunkReader for Woff1Font {
    type ChunkType = WoffChunkType;
    type Error = FontIoError;

    fn get_chunk_positions(
        reader: &mut (impl Read + Seek + ?Sized),
    ) -> Result<Vec<ChunkPosition<Self::ChunkType>>, Self::Error> {
        let woff_header = Woff1Header::from_reader(reader)?;
        let size_to_read =
            woff_header.numTables as usize * Woff1DirectoryEntry::SIZE;
        let directory = Woff1Directory::from_reader_exact(
            reader,
            Woff1Header::SIZE as u64,
            size_to_read,
        )?;

        let mut positions: Vec<ChunkPosition<Self::ChunkType>> = Vec::new();
        positions.push(ChunkPosition::new(
            0,
            Woff1Header::SIZE,
            WOFF_HEADER_CHUNK_NAME.data,
            WoffChunkType::Header,
        ));
        tracing::trace!("Header position information added");
        positions.push(ChunkPosition::new(
            Woff1Header::SIZE,
            size_to_read,
            WOFF_DIRECTORY_CHUNK_NAME.data,
            WoffChunkType::DirectoryEntry,
        ));
        tracing::trace!("Directory position information added");

        // Loop through all of the entries
        for entry in directory.entries() {
            positions.push(ChunkPosition::new(
                entry.offset() as usize,
                entry.length() as usize,
                entry.tag().data,
                WoffChunkType::TableData,
            ));
            tracing::trace!("Table data position information added");
        }

        // If we have metadata, add it
        if woff_header.metaLength > 0 {
            positions.push(ChunkPosition::new(
                woff_header.metaOffset as usize,
                woff_header.metaLength as usize,
                WOFF_METADATA_CHUNK_NAME.data,
                WoffChunkType::Metadata,
            ));
            tracing::trace!("Metadata position information added");
        }

        // If we have private data, add it
        if woff_header.privLength > 0 {
            positions.push(ChunkPosition::new(
                woff_header.privOffset as usize,
                woff_header.privLength as usize,
                WOFF_PRIVATE_DATA_CHUNK_NAME.data,
                WoffChunkType::Private,
            ));
            tracing::trace!("Private data position information added");
        }
        Ok(positions)
    }
}

impl C2PASupport for Woff1Font {
    type Error = FontIoError;

    fn add_c2pa_record(
        &mut self,
        record: crate::c2pa::ContentCredentialRecord,
    ) -> Result<(), Self::Error> {
        // Look for an entry in the table
        match self.tables.entry(FontTag::C2PA) {
            // If we do not have an entry, we are good to go to insert the
            // record
            Entry::Vacant(entry) => {
                // If we don't have an entry, create one
                let table = TableC2PA {
                    major_version: record.major_version(),
                    minor_version: record.minor_version(),
                    active_manifest_uri: record
                        .active_manifest_uri()
                        .map(|s| s.to_owned()),
                    manifest_store: record
                        .content_credential()
                        .map(|s| s.to_vec()),
                };
                entry.insert(NamedTable::C2PA(table));
                Ok(())
            }
            // Otherwise, we are in an error state
            Entry::Occupied(_) => {
                // If we do have an entry, return an error
                Err(FontIoError::ContentCredentialAlreadyExists)
            }
        }
    }

    fn has_c2pa(&self) -> bool {
        self.tables.contains_key(&FontTag::C2PA)
    }

    fn get_c2pa(
        self,
    ) -> Result<Option<crate::c2pa::ContentCredentialRecord>, Self::Error> {
        // We will need to read in the C2PA table, it could be compressed, so we
        // will need to look at the directory entry to see if it is
        // compressed or not. If it is compressed, we will need to
        // decompress it before we can read it.
        let mut tables = self.tables;
        if let Some(NamedTable::C2PA(table)) = tables.get_mut(&FontTag::C2PA) {
            let table: &TableC2PA = &*table;
            let record = crate::c2pa::ContentCredentialRecord::try_from(table)?;
            Ok(Some(record))
        } else {
            // If we don't have a C2PA table, return None
            Ok(None)
        }
    }

    fn remove_c2pa_record(&mut self) -> Result<(), Self::Error> {
        match self.tables.entry(FontTag::C2PA) {
            Entry::Vacant(_) => Err(FontIoError::ContentCredentialNotFound),
            Entry::Occupied(entry) => {
                entry.remove();
                Ok(())
            }
        }
    }
}

#[cfg(test)]
#[path = "font_test.rs"]
mod tests;
