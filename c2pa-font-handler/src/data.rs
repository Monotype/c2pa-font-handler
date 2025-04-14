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

//! A generic data structure for reading and writing data (e.g. OTF/WOFF1
//! tables).

use std::{
    io::{Read, Seek, SeekFrom, Write},
    num::Wrapping,
};

use crate::{
    error::FontIoError, utils, FontDataChecksum, FontDataExactRead,
    FontDataWrite, FontTable,
};

/// Generic data structure for reading and writing data (e.g. OTF/WOFF1 tables).

#[derive(Debug, Default)]
pub struct Data {
    /// The data
    pub(crate) data: Vec<u8>,
}

impl Data {
    /// Create a new Data record with the given data
    pub fn new(data: Vec<u8>) -> Self {
        Data { data }
    }

    /// Set the associated data
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }
}

impl FontDataExactRead for Data {
    type Error = FontIoError;

    fn from_reader_exact<T: Read + Seek + ?Sized>(
        reader: &mut T,
        offset: u64,
        size: usize,
    ) -> Result<Self, Self::Error> {
        reader.seek(SeekFrom::Start(offset))?;
        let mut data = vec![0; size];
        reader.read_exact(&mut data)?;
        Ok(Data { data })
    }
}

impl FontDataWrite for Data {
    type Error = FontIoError;

    fn write<TDest: Write + ?Sized>(
        &self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        dest.write_all(&self.data[..])
            .map_err(FontIoError::FailedToWriteFontData)?;
        let limit = self.data.len() % 4;
        if limit > 0 {
            let padding = vec![0; 4 - limit];
            dest.write_all(&padding[..])
                .map_err(FontIoError::FailedToWriteFontData)?;
        }
        Ok(())
    }
}

impl FontDataChecksum for Data {
    fn checksum(&self) -> Wrapping<u32> {
        utils::checksum(&self.data)
    }
}

impl FontTable for Data {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn len(&self) -> u32 {
        self.data.len() as u32
    }
}

#[cfg(test)]
#[path = "data_test.rs"]
mod tests;
