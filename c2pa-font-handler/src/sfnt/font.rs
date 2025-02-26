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

use std::{
    collections::{btree_map::Entry, BTreeMap},
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
    error::{FontIoError, FontSaveError},
    sfnt::table::TableC2PA,
    tag::FontTag,
    utils::align_to_four,
    Font, FontDSIGStubber, FontDataChecksum, FontDataRead, FontDataWrite,
    FontDirectory, FontHeader, FontTable, MutFontDataWrite,
};

/// Pseudo-tag for the table directory
const _SFNT_DIRECTORY_CHUNK_NAME: FontTag = FontTag { data: *b" DIR" };

/// Implementation of an SFNT font.
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
        self.directory.physical_order().iter().for_each(|entry| {
            // If we have this entry in our current table list, create new entry
            if self.tables.contains_key(&entry.tag) {
                let neo_entry = SfntDirectoryEntry {
                    tag: entry.tag,
                    offset: running_offset,
                    checksum: self.tables[&entry.tag].checksum().0,
                    length: self.tables[&entry.tag].len(),
                };
                neo_directory.add_entry(neo_entry);
                // Update our running offset.
                running_offset += align_to_four(self.tables[&entry.tag].len());
            }
        });

        if let Some(c2pa) = self.tables.get(&FontTag::C2PA) {
            if !self
                .directory
                .entries()
                .iter()
                .any(|entry| entry.tag == FontTag::C2PA)
            {
                let neo_entry = SfntDirectoryEntry {
                    tag: FontTag::C2PA,
                    offset: running_offset,
                    checksum: c2pa.checksum().0,
                    length: c2pa.len(),
                };
                neo_directory.add_entry(neo_entry);
            }
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

#[cfg(test)]
#[path = "font_test.rs"]
mod tests;
