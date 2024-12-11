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

//! Font tag

use std::io::{Read, Seek, Write};

use super::{error::FontIoError, FontDataRead, FontDataWrite};

/// Four-character tag which names a font table
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FontTag {
    /// The four-character tag data
    data: [u8; 4],
}

impl FontTag {
    /// Tag for the Digital Signature table
    pub(crate) const DSIG: FontTag = FontTag { data: *b"DSIG" };
    /// Tag for the 'head' table
    pub(crate) const HEAD: FontTag = FontTag { data: *b"head" };
    /// Size for a `FontTag`
    pub(crate) const SIZE: usize = 4;

    /// Creates a new `SfntTag` from a four-character array.
    pub fn new(source_data: [u8; 4]) -> Self {
        Self { data: source_data }
    }

    /// Returns the four-character tag data.
    pub fn data(&self) -> [u8; 4] {
        self.data
    }
}

impl FontDataRead for FontTag {
    type Error = FontIoError;

    fn from_reader<T: Read + Seek + ?Sized>(
        reader: &mut T,
    ) -> Result<Self, Self::Error> {
        let mut data = [0; Self::SIZE];
        reader.read_exact(&mut data)?;
        Ok(Self::new(data))
    }

    fn from_reader_exact<T: Read + Seek + ?Sized>(
        reader: &mut T,
        offset: u64,
        size: usize,
    ) -> Result<Self, Self::Error> {
        if size != Self::SIZE {
            return Err(FontIoError::InvalidSizeForTAG(size));
        }
        reader.seek(std::io::SeekFrom::Start(offset))?;
        Self::from_reader(reader)
    }
}

impl FontDataWrite for FontTag {
    type Error = FontIoError;

    fn write<TDest: Write + ?Sized>(
        &self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        dest.write_all(&self.data)?;
        Ok(())
    }
}

impl std::fmt::Display for FontTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.data))
    }
}

impl std::fmt::Debug for FontTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FontTag({})", self)
    }
}

#[cfg(test)]
#[path = "tag_test.rs"]
mod tests;
