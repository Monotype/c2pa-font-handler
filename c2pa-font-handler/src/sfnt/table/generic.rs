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

//! Generic/Unspecified SFNT table.
use std::io::{Read, Seek, SeekFrom, Write};

use crate::{
    error::FontIoError, utils, FontDataChecksum, FontDataRead, FontDataWrite,
    FontTable,
};

/// Generic font table with unknown contents.
pub(crate) struct TableGeneric {
    pub data: Vec<u8>,
}

impl FontDataRead for TableGeneric {
    type Error = FontIoError;

    fn from_reader<T: Read + Seek + ?Sized>(
        reader: &mut T,
    ) -> Result<Self, Self::Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        Ok(TableGeneric { data })
    }

    fn from_reader_exact<T: Read + Seek + ?Sized>(
        reader: &mut T,
        offset: u64,
        size: usize,
    ) -> Result<Self, Self::Error> {
        reader.seek(SeekFrom::Start(offset))?;
        let mut data = vec![0; size];
        reader.read_exact(&mut data)?;

        Ok(TableGeneric { data })
    }
}

impl FontDataWrite for TableGeneric {
    type Error = FontIoError;

    fn write<TDest: Write + ?Sized>(
        &self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error> {
        // write all of the data to the destination
        dest.write_all(&self.data[..])
            .map_err(FontIoError::FailedToWriteTableData)?;
        // And determine the padding needed to be byte aligned
        let limit = self.data.len() % 4;
        if limit > 0 {
            let padding = vec![0; 4 - limit];
            dest.write_all(&padding)
                .map_err(FontIoError::FailedToWriteTableData)?;
        }
        Ok(())
    }
}

impl FontDataChecksum for TableGeneric {
    fn checksum(&self) -> std::num::Wrapping<u32> {
        utils::checksum(&self.data)
    }
}

impl FontTable for TableGeneric {
    fn len(&self) -> u32 {
        self.data.len() as u32
    }
}

#[cfg(test)]
#[path = "generic_test.rs"]
mod tests;
