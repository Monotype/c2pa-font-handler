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

//! Named table enumeration for WOFF fonts

use std::io::{Read, Seek, Write};

use crate::{
    data::Data, error::FontIoError, sfnt::table::TableC2PA, tag::FontTag,
    FontDataChecksum, FontDataExactRead, FontDataWrite, FontTable,
};

/// Various types of tables by name
pub enum NamedTable {
    /// 'C2PA' table
    C2PA(TableC2PA),
    /// Compressed 'C2PA' table
    CompressedC2PA(Data),
    /// Generic table
    Generic(Data),
}

impl NamedTable {
    /// Creates a new `NamedTable` from a reader.
    pub fn from_reader_exact<T: Read + Seek + ?Sized>(
        tag: &FontTag,
        reader: &mut T,
        offset: u64,
        size: usize,
    ) -> Result<Self, FontIoError> {
        match *tag {
            FontTag::C2PA => TableC2PA::from_reader_exact(reader, offset, size)
                .map(NamedTable::C2PA),
            _ => Data::from_reader_exact(reader, offset, size)
                .map(NamedTable::Generic),
        }
    }
}

impl FontDataWrite for NamedTable {
    type Error = FontIoError;

    fn write<TDest: Write + ?Sized>(
        &self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        match self {
            NamedTable::C2PA(table) => table.write(dest)?,
            NamedTable::CompressedC2PA(table) => table.write(dest)?,
            NamedTable::Generic(table) => table.write(dest)?,
        }
        Ok(())
    }
}

impl FontDataChecksum for NamedTable {
    fn checksum(&self) -> std::num::Wrapping<u32> {
        match self {
            NamedTable::C2PA(table) => table.checksum(),
            NamedTable::CompressedC2PA(table) => table.checksum(),
            NamedTable::Generic(table) => table.checksum(),
        }
    }
}

impl FontTable for NamedTable {
    fn len(&self) -> u32 {
        match self {
            NamedTable::C2PA(table) => table.len(),
            NamedTable::CompressedC2PA(table) => table.len(),
            NamedTable::Generic(table) => table.len(),
        }
    }
}

#[cfg(test)]
#[path = "named_table_test.rs"]
mod tests;
