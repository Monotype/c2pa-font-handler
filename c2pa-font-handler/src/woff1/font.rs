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
    io::{Cursor, Read, Seek, SeekFrom, Write},
};

use super::{
    directory::{Woff1Directory, Woff1DirectoryEntry},
    header::Woff1Header,
    table::NamedTable,
};
use crate::{
    c2pa::{C2PASupport, UpdatableC2PA, UpdateContentCredentialRecord},
    chunks::{ChunkPosition, ChunkReader, ChunkTypeTrait},
    compression::{CompressingWriter, DecompressingReader},
    data::Data,
    error::FontIoError,
    sfnt::{
        directory::SfntDirectoryEntry, header::SfntHeader, table::TableC2PA,
    },
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

impl Woff1Font {
    /// Read and decompress a table from the WOFF1 font, for the
    /// given directory entry.
    pub(crate) fn decompress_table_from_stream<R: Read + Seek + ?Sized>(
        entry: &Woff1DirectoryEntry,
        reader: &mut R,
    ) -> Result<NamedTable, FontIoError> {
        tracing::trace!("Decompressing table: {:?}", entry.tag());
        // Seek to the start of the compressed data
        reader.seek(SeekFrom::Start(entry.offset as u64))?;

        // Grab a portion of the stream which will only read the size of
        // the compressed data.
        let mut stream_slice = reader.take(entry.compLength as u64);
        // Create a decompressing reader
        let mut decompress_reader =
            DecompressingReader::builder(&mut stream_slice).build();

        // Read decompressed data into a buffer
        let mut decompressed_data = vec![0; entry.origLength as usize];
        decompress_reader.read_exact(&mut decompressed_data)?;
        // Use a Cursor to wrap the decompressed data
        let mut cursor = Cursor::new(decompressed_data);

        let table = NamedTable::from_reader_exact(
            &entry.tag(),
            &mut cursor,
            0,
            entry.origLength as usize,
        )?;
        tracing::trace!("Decompressed table for entry: {:?}", entry,);
        Ok(table)
    }

    /// Optimizes the table data by compressing it if it is larger than
    /// the original data. If the compressed data is larger than the
    /// original data, the original data is returned.
    fn optimize_table_data<R: Read + Seek + ?Sized>(
        reader: &mut R,
        offset: u64,
        length: u32,
    ) -> Result<WoffTableData, FontIoError> {
        // Seek to the position we are to read from
        reader.seek(SeekFrom::Start(offset))?;

        // Create a buffer to hold the uncompressed data
        let mut uncompressed_data = vec![0; length as usize];
        reader.read_exact(&mut uncompressed_data)?;

        // Create a buffer to hold the compressed data
        let mut compressed_data = Vec::new();
        {
            let mut compressed_writer =
                CompressingWriter::builder(&mut compressed_data).build();
            compressed_writer.write_all(&uncompressed_data)?;
            compressed_writer.finish()?; // ensure all data is written
        }
        let compressed_length = compressed_data.len() as u32;

        // Build up the return value based on if we actually saved space
        // compressing
        if compressed_length >= length {
            tracing::debug!("Not compressing C2PA table");
            // If we didn't save space, just return the original data
            Ok(WoffTableData::Uncompressed {
                data: Data::new(uncompressed_data),
                length,
            })
        } else {
            tracing::debug!(
                "Compressing C2PA table; saved {} bytes",
                length - compressed_length
            );
            Ok(WoffTableData::Compressed {
                data: Data::new(compressed_data),
                compressed_length,
                original_length: length,
            })
        }
    }

    /// Prepare a new header based on the current state of the font.
    fn prepare_header(&self) -> Woff1Header {
        // Fill in the new header with the old header's values
        Woff1Header {
            flavor: self.header.flavor,
            length: self.header.length,
            numTables: self.tables.len() as u16,
            reserved: self.header.reserved,
            totalSfntSize: self.header.totalSfntSize,
            majorVersion: self.header.majorVersion,
            minorVersion: self.header.minorVersion,
            metaOffset: self.header.metaOffset,
            metaLength: self.header.metaLength,
            metaOrigLength: self.header.metaOrigLength,
            privOffset: self.header.privOffset,
            privLength: self.header.privLength,
            ..Default::default()
        }
    }
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

        for entry in directory.entries() {
            // check if the entry is compressed
            let table = if entry.compLength < entry.origLength
                && entry.tag == FontTag::C2PA
            {
                Self::decompress_table_from_stream(entry, reader)?
            } else {
                // Read in the table data
                NamedTable::from_reader_exact(
                    &entry.tag(),
                    reader,
                    entry.offset as u64,
                    entry.length() as usize,
                )?
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
        let mut neo_header = self.prepare_header();
        let mut neo_directory = Woff1Directory::default();

        // Fill in the new directory with the old directory's values
        let new_table_count = self.tables.len() as u16;

        // Use a running offset to calculate the new offsets
        let mut running_offset = Woff1Header::SIZE as u32
            + new_table_count as u32 * Woff1DirectoryEntry::SIZE as u32;

        // Iterate over the old directory and add entries to the new directory
        self.directory
            .physical_order()
            .iter()
            .filter(|entry| entry.tag != FontTag::C2PA)
            .for_each(|entry| {
                // If we have a table for the entry, add it to the new directory
                if let Some(table) = self.tables.get(&entry.tag) {
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

        // We will need to keep up with the original checksum for the C2PA table
        // to write it later in the directory entry
        let mut original_checksum = 0;
        // If we have a C2PA table, we will attempt to compress it
        let c2pa_data = self
            .tables
            .get(&FontTag::C2PA)
            .map(|c2pa| {
                original_checksum = c2pa.checksum().0;
                let mut data_to_compress = Vec::new();
                c2pa.write(&mut data_to_compress)?;
                let c2pa_table = Self::optimize_table_data(
                    &mut Cursor::new(data_to_compress),
                    0,
                    c2pa.len(),
                )?;
                // Add the C2PA table to the new directory
                neo_directory.add_entry(Woff1DirectoryEntry {
                    tag: FontTag::C2PA,
                    offset: running_offset,
                    compLength: c2pa_table.compressed_length(),
                    origLength: c2pa_table.length(),
                    origChecksum: original_checksum,
                });

                running_offset += align_to_four(c2pa_table.compressed_length());
                Ok::<_, FontIoError>(c2pa_table)
            })
            .transpose()?;

        // Sort the new directory by tag
        neo_directory.sort_entries(|entry| entry.tag);

        // If we have extension metadata, update the header
        if let Some(meta) = &self.metadata {
            neo_header.metaOffset = running_offset;
            let meta_length = meta.len();
            neo_header.metaLength = meta_length;
            running_offset += align_to_four(meta_length);
        }

        // If we have private data, update the header
        if let Some(private) = &self.private_data {
            let private_length = private.len();
            neo_header.privOffset = running_offset;
            neo_header.privLength = private_length;
            running_offset += align_to_four(private_length);
        }

        // Update the header with the new length of the entire file
        neo_header.length = running_offset;

        neo_header.totalSfntSize = {
            let mut total_sfnt_size = SfntHeader::SIZE as u32; // Size of SFNT header
            total_sfnt_size +=
                new_table_count as u32 * SfntDirectoryEntry::SIZE as u32; // Size of table record (directory of font tables)
                                                                          // Add the size of each table in the directory
            for table in neo_directory.entries() {
                total_sfnt_size += align_to_four(table.origLength);
            }
            total_sfnt_size
        };

        // Update the number of tables in the header, which will include the
        // C2PA table
        neo_header.numTables = new_table_count;

        // Update ourselves with the new header and directory
        self.header = neo_header;
        self.directory = neo_directory;

        // Write the header and directory
        self.header.write(dest)?;
        self.directory.write(dest)?;
        // And write out the tables
        for entry in self.directory.physical_order() {
            match entry.tag {
                FontTag::C2PA => {
                    if let Some(c2pa) = &c2pa_data {
                        c2pa.data().write(dest)?;
                    } else {
                        // Weird case, because if we have a C2PA table, we
                        // should have a C2PA entry in the directory
                        tracing::error!(
                            "C2PA table not found (this should not happen)"
                        );
                        return Err(FontIoError::ContentCredentialNotFound);
                    }
                }
                _ => self.tables[&entry.tag].write(dest)?,
            }
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
            WoffChunkType::Header | WoffChunkType::DirectoryEntry => false,
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

impl UpdatableC2PA for Woff1Font {
    type Error = FontIoError;

    fn update_c2pa_record(
        &mut self,
        record: UpdateContentCredentialRecord,
    ) -> Result<(), Self::Error> {
        // Look for an entry in the table
        match self.tables.entry(FontTag::C2PA) {
            // If the entry is vacant, insert a new C2PA table
            Entry::Vacant(vacant_entry) => {
                tracing::debug!("Adding C2PA table");
                let mut c2pa_table = TableC2PA::default();
                c2pa_table.update_c2pa_record(record)?;
                vacant_entry.insert(NamedTable::C2PA(c2pa_table));
                Ok(())
            }
            // Otherwise, we already have a record, so we need to update it
            Entry::Occupied(mut occupied_entry) => {
                tracing::debug!("Updating C2PA table");
                match occupied_entry.get_mut() {
                    NamedTable::C2PA(table_c2pa) => {
                        table_c2pa.update_c2pa_record(record)?;
                        Ok(())
                    }
                    // This technically should not happen, but we will
                    // take it into account.
                    other => {
                        tracing::error!("C2PA tag exists but is not a C2PA table: found {other}");
                        Err(FontIoError::InvalidC2paTableContainer)
                    }
                }
            }
        }
    }
}

/// The data for a table in the WOFF1 font
enum WoffTableData {
    /// Compressed data
    Compressed {
        data: Data,
        compressed_length: u32,
        original_length: u32,
    },
    /// Uncompressed data
    Uncompressed { data: Data, length: u32 },
}

impl WoffTableData {
    /// Get the length of the data
    fn length(&self) -> u32 {
        match self {
            WoffTableData::Compressed {
                original_length, ..
            } => *original_length,
            WoffTableData::Uncompressed { length, .. } => *length,
        }
    }

    /// Get the compressed length of the data
    fn compressed_length(&self) -> u32 {
        match self {
            WoffTableData::Compressed {
                compressed_length, ..
            } => *compressed_length,
            // For WOFF, the compressed length is the same as the uncompressed
            // length, so we return the uncompressed length
            WoffTableData::Uncompressed { length, .. } => *length,
        }
    }

    /// Get the data
    fn data(&self) -> &Data {
        match self {
            WoffTableData::Compressed { data, .. } => data,
            WoffTableData::Uncompressed { data, .. } => data,
        }
    }
}

#[cfg(test)]
#[path = "font_test.rs"]
mod tests;
