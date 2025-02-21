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

//! WOFF1 extension metadata

use std::num::Wrapping;

use crate::{
    error::FontIoError, utils, FontDataChecksum, FontDataExactRead,
    FontDataWrite, FontTable,
};

/// WOFF1 extension metadata

#[derive(Debug, Default)]
pub struct Data {
    /// The data from the metadata block
    pub(crate) data: Vec<u8>,
}

impl Data {
    /// Create a new Data record with the given data
    pub fn new(data: Vec<u8>) -> Self {
        Data { data }
    }

    /// Get the data associated with the metadata
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Set the data associated with the metadata
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }
}

impl FontDataExactRead for Data {
    type Error = crate::error::FontIoError;

    fn from_reader_exact<T: std::io::Read + std::io::Seek + ?Sized>(
        reader: &mut T,
        _offset: u64,
        size: usize,
    ) -> Result<Self, Self::Error> {
        let mut data = vec![0; size];
        reader.read_exact(&mut data)?;
        Ok(Data { data })
    }
}

impl FontDataWrite for Data {
    type Error = crate::error::FontIoError;

    fn write<TDest: std::io::Write + ?Sized>(
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
    fn len(&self) -> u32 {
        self.data.len() as u32
    }
}

#[cfg(test)]
#[path = "data_test.rs"]
mod tests;
