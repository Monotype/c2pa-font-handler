// Copyright 2024 Monotype Imaging Inc.
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

//! Named table enumeration.
use std::io::{Read, Seek, Write};

use super::{dsig::TableDSIG, generic::TableGeneric, head::TableHead};
use crate::{
    error::FontIoError, tag::FontTag, FontDataChecksum, FontDataRead,
    FontDataWrite, FontTable,
};

/// Various types of tables by name
pub(crate) enum NamedTable {
    #[allow(clippy::upper_case_acronyms)]
    DSIG(TableDSIG),
    Head(TableHead),
    Generic(TableGeneric),
}

impl NamedTable {
    pub fn from_reader_exact<T: Read + Seek + ?Sized>(
        tag: &FontTag,
        reader: &mut T,
        offset: u64,
        size: usize,
    ) -> Result<Self, FontIoError> {
        match *tag {
            FontTag::DSIG => TableDSIG::from_reader_exact(reader, offset, size)
                .map(NamedTable::DSIG),
            FontTag::HEAD => TableHead::from_reader_exact(reader, offset, size)
                .map(NamedTable::Head),
            _ => TableGeneric::from_reader_exact(reader, offset, size)
                .map(NamedTable::Generic),
        }
    }

    pub fn len(&self) -> u32 {
        match self {
            NamedTable::DSIG(table) => table.len(),
            NamedTable::Head(table) => table.len(),
            NamedTable::Generic(table) => table.len(),
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
            NamedTable::DSIG(table) => table.write(dest)?,
            NamedTable::Head(table) => table.write(dest)?,
            NamedTable::Generic(table) => table.write(dest)?,
        }
        Ok(())
    }
}

impl FontDataChecksum for NamedTable {
    fn checksum(&self) -> std::num::Wrapping<u32> {
        match self {
            NamedTable::DSIG(table) => table.checksum(),
            NamedTable::Head(table) => table.checksum(),
            NamedTable::Generic(table) => table.checksum(),
        }
    }
}

#[cfg(test)]
#[path = "named_table_test.rs"]
mod tests;
