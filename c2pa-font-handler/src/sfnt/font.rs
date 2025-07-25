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

//! SFNT font.
//!
//! The crate provides an implementation of the SFNT font format, which is
//! used by OpenType and TrueType fonts. It includes functionality for reading
//! and writing SFNT fonts, as well as stubbing the DSIG table.
//!
//! # Stubbing the DSIG Table
//!
//! The DSIG table is used to store digital signatures for fonts. This crate
//! provides functionality to stub the DSIG table, which is useful for
//! keeping a DSIG if present, but zeroing out the signatures and just having
//! a stub table. This is useful for fonts that are being modified or
//! processed (i.e., when adding C2PA), as it allows the font to be saved
//! without invalidating the signatures.
//!
//! The following is an example of how to efficiently stub the DSIG table in a
//! stream without loading the entire font into memory:
//!
//! ```no_run
//! use std::{
//!     fs::File,
//!     io::{BufReader, BufWriter},
//! };
//!
//! use c2pa_font_handler::{error::FontIoError, sfnt::font::stub_dsig_stream};
//! # fn main() -> Result<(), FontIoError> {
//! let input_file = File::open("path/to/input/font.ttf")?;
//! let output_file = File::create("path/to/output/font.ttf")?;
//! let mut reader = BufReader::new(input_file);
//! let mut writer = BufWriter::new(output_file);
//! stub_dsig_stream(&mut reader, &mut writer)?;
//! # Ok(())
//! # }
//! ```

use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Display,
    io::{Read, Seek},
    num::Wrapping,
};

use super::{
    directory::{SfntDirectory, SfntDirectoryEntry},
    header::SfntHeader,
    table::{
        dsig::TableDSIG, head::SFNT_EXPECTED_CHECKSUM, named_table::NamedTable,
    },
};
use crate::{
    c2pa::{C2PASupport, UpdatableC2PA},
    chunks::{ChunkPosition, ChunkReader, ChunkTypeTrait},
    error::{FontIoError, FontSaveError},
    sfnt::table::TableC2PA,
    tag::FontTag,
    utils::align_to_four,
    DSIGType, Font, FontDSIGDetector, FontDSIGStubber, FontDataChecksum,
    FontDataExactRead, FontDataRead, FontDataWrite, FontDirectory,
    FontDirectoryEntry, FontHeader, FontTable, MutFontDataWrite,
};

/// Implementation of an SFNT font.
///
/// # Remarks
/// If the 'woff' feature is enabled, this type can also be created from a
/// a Woff1Font using the `TryFrom` trait. This is not intended to be used
/// to produce production-ready SFNT fonts, but serves as a way to utilize
/// thumbnails for WOFF fonts.
#[derive(Default)]
pub struct SfntFont {
    header: SfntHeader,
    directory: SfntDirectory,
    tables: BTreeMap<FontTag, NamedTable>,
}

impl FontDataRead for SfntFont {
    type Error = FontIoError;

    fn from_reader<T: Read + Seek + ?Sized>(
        reader: &mut T,
    ) -> Result<Self, Self::Error> {
        let header = SfntHeader::from_reader(reader)?;
        let directory = SfntDirectory::from_reader_with_count(
            reader,
            header.num_tables() as usize,
        )?;
        let mut tables = BTreeMap::new();
        for entry in directory.entries() {
            let table = NamedTable::from_reader_exact(
                &entry.tag,
                reader,
                entry.offset as u64,
                entry.length as usize,
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

impl MutFontDataWrite for SfntFont {
    type Error = FontIoError;

    fn write<TDest: std::io::Write + ?Sized>(
        &mut self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        let mut neo_header = SfntHeader::default();
        let mut neo_directory = SfntDirectory::new();
        // Re-synthesize the file header based on the actual table count
        neo_header.sfntVersion = self.header.sfntVersion;
        neo_header.numTables = self.tables.len() as u16;
        neo_header.entrySelector = if neo_header.numTables > 0 {
            neo_header.numTables.ilog2() as u16
        } else {
            return Err(FontSaveError::NoTablesFound.into());
        };
        neo_header.searchRange =
            2_u16.pow(neo_header.entrySelector as u32) * 16;
        neo_header.rangeShift =
            neo_header.numTables * 16 - neo_header.searchRange;

        // Currently we only allow a single C2PA table to be removed or added.
        // Table modifications are allowed.  Verify that this is the case.
        let orig_table_count = self.header.numTables;
        let new_table_count = self.tables.len() as u16;
        let table_diff = new_table_count as i32 - orig_table_count as i32;
        // Make sure we only removed at most one table.
        if table_diff < -1 {
            return Err(FontSaveError::TooManyTablesRemoved.into());
        }
        // Make sure we only added at most one table.
        else if table_diff > 1 {
            return Err(FontSaveError::TooManyTablesAdded.into());
        }

        // Keep a running offset as we encounter our tables in physical order.
        let mut running_offset = SfntHeader::SIZE as u32
            + SfntDirectoryEntry::SIZE as u32 * new_table_count as u32;

        // Walk our old directory in physical order, adding new entries for each
        // table we still have.
        self.directory
            .physical_order()
            .iter()
            .filter(|t| t.tag != FontTag::C2PA) // C2PA should always be at the end
            .for_each(|entry| {
                // If we have this entry in our current table list, create new
                // entry
                if self.tables.contains_key(&entry.tag) {
                    let neo_entry = SfntDirectoryEntry {
                        tag: entry.tag,
                        offset: running_offset,
                        checksum: self.tables[&entry.tag].checksum().0,
                        length: self.tables[&entry.tag].len(),
                    };
                    neo_directory.add_entry(neo_entry);
                    // Update our running offset.
                    running_offset +=
                        align_to_four(self.tables[&entry.tag].len());
                }
            });

        if let Some(c2pa) = self.tables.get(&FontTag::C2PA) {
            let neo_entry = SfntDirectoryEntry {
                tag: FontTag::C2PA,
                offset: running_offset,
                checksum: c2pa.checksum().0,
                length: c2pa.len(),
            };
            neo_directory.add_entry(neo_entry);
        }

        // Sort our directory entries by tag.
        neo_directory.sort_entries(|entry| entry.tag);

        // Figure the checksum for the whole font - the header, the directory,
        // and then all the tables; we can just use the per-table checksums,
        // since the only one we alter is C2PA, and we just refreshed it...
        let font_cksum = neo_header.checksum()
            + neo_directory.checksum()
            + neo_directory
                .entries()
                .iter()
                .fold(Wrapping(0_u32), |tables_cksum, entry| {
                    tables_cksum + Wrapping(entry.checksum)
                });

        // Rewrite the head table's checksumAdjustment. (This act does *not*
        // invalidate the checksum in the TDE for the 'head' table, which is
        // always treated as zero during check summing).
        if let Some(NamedTable::Head(head)) =
            self.tables.get_mut(&FontTag::HEAD)
        {
            head.checksumAdjustment =
                (Wrapping(SFNT_EXPECTED_CHECKSUM) - font_cksum - Wrapping(0)).0;
        }

        // Replace our header & directory with updated editions.
        self.header = neo_header;
        self.directory = neo_directory;
        // Write everything out.
        self.header.write(dest)?;
        self.directory.write(dest)?;
        for entry in self.directory.physical_order().iter() {
            self.tables[&entry.tag].write(dest)?;
        }
        Ok(())
    }
}

impl FontDSIGStubber for SfntFont {
    type Error = FontIoError;

    fn stub_dsig(&mut self) -> Result<(), Self::Error> {
        if let Entry::Occupied(mut entry) = self.tables.entry(FontTag::DSIG) {
            // Create the stub DSIG table.
            let dsig_table = NamedTable::DSIG(TableDSIG::stub());
            // Replace the DSIG table with a minimal version.
            entry.insert(dsig_table);
        }
        Ok(())
    }
}

impl<T: Read + Seek + ?Sized> FontDSIGDetector for T {
    type Error = FontIoError;

    fn check_for_dsig(&mut self) -> Result<crate::DSIGType, Self::Error> {
        // Grab the original position.
        let original_position = self.stream_position()?;
        // We need to parse the header to be able to read the table directory.
        let font_header = SfntHeader::from_reader(self)?;
        // And now we can read the table directory.
        let font_directory = SfntDirectory::from_reader_with_count(
            self,
            font_header.numTables as usize,
        )?;
        let dsig_type = match font_directory
            .entries()
            .iter()
            .find(|e| e.tag == FontTag::DSIG)
        {
            Some(entry) => {
                // Since DSIG table, according to the spec, must be at the end
                // of the file we can use the offset to
                // determine where the end of the font data is.
                let original_dsig_offset = entry.offset();
                let dsig_table = TableDSIG::from_reader_exact(
                    self,
                    original_dsig_offset as u64,
                    entry.length() as usize,
                )?;
                if dsig_table.is_stubbed() {
                    tracing::debug!("DSIG table is stubbed.");
                    DSIGType::Stubbed
                } else {
                    tracing::debug!("DSIG table is present and not stubbed.");
                    // If it is not stubbed, we can return that it is present.
                    DSIGType::Present
                }
            }
            None => {
                tracing::debug!("DSIG table is not present.");
                DSIGType::NotPresent
            }
        };
        self.seek(std::io::SeekFrom::Start(original_position))?;
        Ok(dsig_type)
    }
}

/// A convenience function to stub the DSIG table in a stream. This will
/// read the stream, check for the DSIG table, and if it is present, stub
/// it. If the DSIG table is not present or already stubbed, it will simply
/// copy the stream to the writer without modification.
pub fn stub_dsig_stream<R: Read + Seek + ?Sized, W: std::io::Write + ?Sized>(
    reader: &mut R,
    writer: &mut W,
) -> Result<(), FontIoError> {
    match reader.check_for_dsig()? {
        DSIGType::NotPresent | DSIGType::Stubbed => {
            tracing::debug!(
                "DSIG table is not present or already stubbed, copying stream."
            );
            std::io::copy(reader, writer)?;
        }
        DSIGType::Present => {
            tracing::debug!(
                "DSIG table is present and not stubbed, proceeding to stub it."
            );
            let mut sfnt_font = SfntFont::from_reader(reader)?;
            sfnt_font.stub_dsig()?;
            sfnt_font.write(writer)?;
        }
    };
    Ok(())
}

impl C2PASupport for SfntFont {
    type Error = FontIoError;

    fn add_c2pa_record(
        &mut self,
        record: crate::c2pa::ContentCredentialRecord,
    ) -> Result<(), Self::Error> {
        // Look for an entry int he table
        match self.tables.entry(FontTag::C2PA) {
            // if vacant, we are good to go to insert the record
            Entry::Vacant(vacant_entry) => {
                let c2pa_table = TableC2PA {
                    major_version: record.major_version(),
                    minor_version: record.minor_version(),
                    active_manifest_uri: record
                        .active_manifest_uri()
                        .map(|s| s.to_owned()),
                    manifest_store: record
                        .content_credential()
                        .map(|s| s.to_vec()),
                };
                vacant_entry.insert(NamedTable::C2PA(c2pa_table));
                Ok(())
            }
            // Otherwise, we are in an error state
            Entry::Occupied(_occupied_entry) => {
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
        if let Some(NamedTable::C2PA(table)) = self.tables.get(&FontTag::C2PA) {
            let record = crate::c2pa::ContentCredentialRecord::try_from(table)?;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    fn remove_c2pa_record(&mut self) -> Result<(), Self::Error> {
        match self.tables.entry(FontTag::C2PA) {
            Entry::Vacant(_vacant_entry) => {
                Err(FontIoError::ContentCredentialNotFound)
            }
            Entry::Occupied(occupied_entry) => {
                occupied_entry.remove();
                Ok(())
            }
        }
    }
}

impl UpdatableC2PA for SfntFont {
    type Error = FontIoError;

    fn update_c2pa_record(
        &mut self,
        record: crate::c2pa::UpdateContentCredentialRecord,
    ) -> Result<(), Self::Error> {
        // Look for an entry int he table
        match self.tables.entry(FontTag::C2PA) {
            // if vacant, we are good to go to insert the record
            Entry::Vacant(vacant_entry) => {
                let mut c2pa_table = TableC2PA::default();
                c2pa_table.update_c2pa_record(record)?;
                vacant_entry.insert(NamedTable::C2PA(c2pa_table));
                Ok(())
            }
            // Otherwise, we already have a record, so we need to update it
            Entry::Occupied(mut occupied_entry) => {
                match occupied_entry.get_mut() {
                    NamedTable::C2PA(table_c2pa) => {
                        table_c2pa.update_c2pa_record(record)?;
                        Ok(())
                    }
                    _ => Err(FontIoError::ContentCredentialNotFound),
                }
            }
        }
    }
}

impl Font for SfntFont {
    type Directory = SfntDirectory;
    type Header = SfntHeader;
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

// Used to indicate the header chunks
const SFNT_HEADER_CHUNK_NAME: FontTag = FontTag { data: *b" HDR" };

/// Chunk types for SFNT fonts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SfntChunkType {
    /// Checksum adjustment
    ChecksumAdjustment,
    /// Header/Directory
    HeaderDirectory,
    /// C2PA table
    C2paTableData,
    /// Table data
    TableData,
}
impl Display for SfntChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SfntChunkType::C2paTableData => write!(f, "C2PA Table Data"),
            SfntChunkType::ChecksumAdjustment => {
                write!(f, "Checksum Adjustment")
            }
            SfntChunkType::HeaderDirectory => write!(f, "HeaderDirectory"),
            SfntChunkType::TableData => write!(f, "Table Data"),
        }
    }
}

impl ChunkTypeTrait for SfntChunkType {
    /// The Header, Directory, ChecksumAdjustment (in the head table), and C2PA
    /// table data should not be hashed; all others should be hashed.
    fn should_hash(&self) -> bool {
        match self {
            SfntChunkType::C2paTableData => true,
            SfntChunkType::ChecksumAdjustment => false,
            SfntChunkType::HeaderDirectory => false,
            SfntChunkType::TableData => true,
        }
    }
}

impl ChunkReader for SfntFont {
    type ChunkType = SfntChunkType;
    type Error = FontIoError;

    fn get_chunk_positions(
        reader: &mut (impl Read + Seek + ?Sized),
    ) -> Result<Vec<ChunkPosition<Self::ChunkType>>, Self::Error> {
        let header = SfntHeader::from_reader(reader)?;
        // Calculate the size to read for the directory
        let size_to_read = header.numTables as usize * SfntDirectoryEntry::SIZE;
        // Get the stream offset
        let offset = reader.stream_position()?;
        // Read in the directory from the font
        let directory =
            SfntDirectory::from_reader_exact(reader, offset, size_to_read)?;

        let mut positions = Vec::new();
        // Push the header information
        positions.push(ChunkPosition::new(
            0,
            SfntHeader::SIZE + size_to_read,
            SFNT_HEADER_CHUNK_NAME.data,
            SfntChunkType::HeaderDirectory,
        ));
        tracing::trace!("HeaderDirectory position information added");

        // And then go through each table entry and calculate the positions of
        // the table data.
        for entry in directory.physical_order() {
            match entry.tag() {
                FontTag::C2PA => {
                    tracing::trace!(
                        "C2PA table found, adding positional information"
                    );
                    positions.push(ChunkPosition::new(
                        entry.offset as usize,
                        entry.length as usize,
                        entry.tag().data,
                        SfntChunkType::C2paTableData,
                    ));
                }
                FontTag::HEAD => {
                    tracing::trace!("'head' table found, adding positional information, where excluding the checksum adjustment");
                    positions.push(ChunkPosition::new(
                        entry.offset() as usize,
                        8_usize,
                        *b"hea0",
                        SfntChunkType::TableData,
                    ));
                    positions.push(ChunkPosition::new(
                        entry.offset() as usize + 8,
                        4_usize,
                        *b"hea1",
                        SfntChunkType::ChecksumAdjustment,
                    ));
                    positions.push(ChunkPosition::new(
                        entry.offset() as usize + 12,
                        42_usize,
                        *b"hea2",
                        SfntChunkType::TableData,
                    ));
                }
                _ => {
                    tracing::trace!(
                        "Adding positional information for table data"
                    );
                    positions.push(ChunkPosition::new(
                        entry.offset() as usize,
                        entry.length() as usize,
                        entry.tag().data,
                        SfntChunkType::TableData,
                    ));
                }
            }
        }
        Ok(positions)
    }
}

#[cfg(feature = "woff")]
impl TryFrom<crate::woff1::font::Woff1Font> for SfntFont {
    type Error = FontIoError;

    fn try_from(
        woff: crate::woff1::font::Woff1Font,
    ) -> Result<Self, Self::Error> {
        // These 'use' are done here because of the gated feature for WOFF
        // support.
        use std::collections::BTreeMap;

        use crate::{
            sfnt::table::NamedTable as SfntNamedTable,
            woff1::table::NamedTable as WoffNamedTable,
        };

        // Number of tables in the WOFF font, excluding C2PA (as the C2PA
        // belonged to the WOFF file not the SFNT font).
        let num_tables = woff
            .directory
            .entries()
            .iter()
            .filter(|e| e.tag != FontTag::C2PA)
            .count() as u16;

        // We must have at least one table to convert to SFNT
        if num_tables == 0 {
            return Err(FontIoError::NoTablesFound);
        }

        // According to the WOFF spec, these three fields MUST be
        // calculated based on the number of tables.
        let entry_selector = (num_tables as f64).log2().floor() as u16;
        let search_range =
            2u16.pow(entry_selector as u32) * SfntDirectoryEntry::SIZE as u16;
        let range_shift =
            num_tables * SfntDirectoryEntry::SIZE as u16 - search_range;

        // Copy over fields as appropriate
        let sfnt_header = SfntHeader {
            sfntVersion: woff.header.flavor.try_into()?,
            numTables: num_tables,
            entrySelector: entry_selector,
            rangeShift: range_shift,
            searchRange: search_range,
        };

        // We will build up the SFNT directory and tables from the WOFF
        let mut sfnt_directory = SfntDirectory::new();
        let mut tables = BTreeMap::new();

        // Iterate over the WOFF directory entries and convert them to SFNT
        for entry in woff.directory.entries() {
            // Create a new directory entry for the SFNT font
            let sfnt_entry = SfntDirectoryEntry {
                tag: entry.tag,
                offset: entry.offset, /* This offset is invalid and but when
                                       * Sfnt is written it will update
                                       * correctly. */
                checksum: entry.origChecksum, /* This should still be valid,
                                               * should we we recalculate it? */
                length: entry.origLength,
            };
            // Add the entry to the SFNT directory
            sfnt_directory.add_entry(sfnt_entry);

            // Get the table from the WOFF font
            let woff_table = woff.get_decompressed_table(&entry.tag)?;

            // If the table was compressed in WOFF, decompress it
            match woff_table {
                WoffNamedTable::Generic(table) => {
                    let sfnt_table = SfntNamedTable::Generic(table.clone());
                    tables.insert(entry.tag, sfnt_table);
                } // Add other variants as needed
                WoffNamedTable::C2PA(_table) => {
                    // C2PA table belongs to the WOFF font, so no need to add it
                    // to the SFNT font.
                    tracing::trace!("WOFF C2PA will not be added to SFNT font");
                }
            };
        }

        Ok(Self {
            header: sfnt_header,
            directory: sfnt_directory,
            tables,
        })
    }
}

#[cfg(test)]
#[path = "font_test.rs"]
mod tests;
