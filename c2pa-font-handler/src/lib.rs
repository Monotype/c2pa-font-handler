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

//! Small font I/O handling crate.
//!
//! The only real use at this point, is to read in an SFNT font file and
//! stub the DSIG table. This is useful when you want a DSIG table, but
//! don't really care about the contents of it. For example, if a font
//! is going to be signed with C2PA, the DSIG table conflicts and will be
//! invalidated. Therefore, it is best to put in a stub table.
//!
//! # Example
//! ```
//! use std::io::Cursor;
//! use c2pa_font_handler::error::FontIoError;
//! use c2pa_font_handler::sfnt::font::SfntFont;
//! use c2pa_font_handler::*;
//!
//! fn main() -> Result<(), FontIoError> {
//!    let font_data = include_bytes!("../../.devtools/font.otf");
//!    let mut reader = Cursor::new(font_data);
//!    let mut font = SfntFont::from_reader(&mut reader)?;
//!    assert_eq!(font.header().num_tables(), 11);
//!    assert_eq!(font.directory().physical_order().len(), 11);
//!    // And clear out DSIG
//!    font.stub_dsig()?;
//!
//!    // Save out the file or do something else with it...
//!
//!    Ok(())
//! }

use std::{
    io::{Read, Seek, Write},
    num::Wrapping,
};

use tag::FontTag;

pub mod c2pa;
pub mod chunks;
#[cfg(feature = "compression")]
pub mod compression;
pub mod data;
pub mod error;
pub(crate) mod magic;
pub mod mime_type;
pub mod sfnt;
pub mod tag;
#[cfg(feature = "thumbnails")]
pub mod thumbnail;
pub(crate) mod utils;
#[cfg(feature = "woff")]
pub mod woff1;

/// Trait for computing a checksum on SFNT data.
pub trait FontDataChecksum {
    /// Computes the checksum for the SFNT data.
    fn checksum(&self) -> Wrapping<u32>;
}

/// Trait for reading SFNT data from a reader.
pub trait FontDataRead
where
    Self: Sized,
{
    /// The error type for reading the data.
    type Error;
    /// Reads the font data from a reader.
    fn from_reader<T: Read + Seek + ?Sized>(
        reader: &mut T,
    ) -> Result<Self, Self::Error>;
}

/// Trait for reading SFNT data from a reader, with exact size information.
pub trait FontDataExactRead
where
    Self: Sized,
{
    /// The error type for reading the data.
    type Error;

    /// Reads the font data from a reader, starting at a specific offset and
    /// reading a specific length.
    fn from_reader_exact<T: Read + Seek + ?Sized>(
        reader: &mut T,
        offset: u64,
        size: usize,
    ) -> Result<Self, Self::Error>;
}

/// Trait for writing SFNT data to a writer.
pub trait FontDataWrite {
    /// The error type for writing the data.
    type Error;
    /// Writes the SFNT data to a writer.
    fn write<TDest: Write + ?Sized>(
        &self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error>;
}

/// Trait for writing SFNT data to a writer, with the ability to modify the
/// object.
pub trait MutFontDataWrite {
    /// The error type for writing the data.
    type Error;
    /// Writes the SFNT data to a writer.
    fn write<TDest: Write + ?Sized>(
        &mut self,
        dest: &mut TDest,
    ) -> Result<(), Self::Error>;
}

/// A font header.
pub trait FontHeader: FontDataRead + FontDataChecksum + FontDataWrite {
    /// Returns the number of tables in the font.
    fn num_tables(&self) -> u16;
}

/// A directory in a font.
pub trait FontDirectory:
    FontDataExactRead + FontDataChecksum + FontDataWrite
{
    /// The type of entry in the directory.
    type Entry: FontDirectoryEntry;
    /// Reads the font directory from a reader, with a specified number of
    /// entries.
    fn from_reader_with_count<T: Read + Seek + ?Sized>(
        reader: &mut T,
        entry_count: usize,
    ) -> Result<Self, <Self as FontDataExactRead>::Error>;
    /// Returns a reference to the entries in this directory.
    fn entries(&self) -> &[Self::Entry];
    /// Returns an array which contains the indices of this directory's entries,
    /// arranged in increasing order of `offset` field.
    fn physical_order(&self) -> Vec<&Self::Entry>;
}

/// A directory entry in a font directory.
pub trait FontDirectoryEntry:
    FontDataRead + FontDataChecksum + FontDataWrite
{
    /// Returns the tag of the table.
    fn tag(&self) -> FontTag;
    /// Returns the checksum of the table data.
    fn data_checksum(&self) -> u32;
    /// Returns the offset of the table.
    fn offset(&self) -> u32;
    /// Returns the length of the table.
    fn length(&self) -> u32;
}

/// A table in a font.
#[allow(clippy::len_without_is_empty)] // Doesn't make sense for this trait to have is_empty.
pub trait FontTable: FontDataChecksum + FontDataWrite {
    /// Returns the length of the table.
    fn len(&self) -> u32;
    /// Returns whether the table is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A trait for getting a reader for a font table.
pub trait FontTableReader<'a> {
    /// The error type for reading the table.
    type Error: Into<crate::error::FontIoError>;
    /// Returns a reader for the font table data.
    fn get_reader(&'a self) -> Result<impl Read + Seek + 'a, Self::Error>;
}

/// Represents a font.
pub trait Font: FontDataRead + MutFontDataWrite {
    /// The header type for the font.
    type Header: FontHeader;
    /// The directory type for the font.
    type Directory: FontDirectory;
    /// The table type for the font.
    type Table: FontTable;
    /// Checks if the font contains a specific table.
    fn contains_table(&self, tag: &FontTag) -> bool;
    /// Returns a specific table from the font.
    fn table(&self, tag: &FontTag) -> Option<&Self::Table>;
    /// Returns the font header.
    fn header(&self) -> &Self::Header;
    /// Returns the font directory.
    fn directory(&self) -> &Self::Directory;
}

/// A trait for stubbing the DSIG table in a font. By this, we mean that the
/// DSIG is stripped and replaced with a very minimal version that is still
/// valid.
pub trait FontDSIGStubber {
    /// The error type for stubbing the DSIG table.
    type Error;
    /// Stub the DSIG table in the font.
    fn stub_dsig(&mut self) -> Result<(), Self::Error>;
}

/// Represents the state of the DSIG table in a font.
pub enum DSIGType {
    /// The DSIG table is not present in the font.
    NotPresent,
    /// The DSIG table is present in the font and valid.
    Present,
    /// The DSIG table is present in the font, but it is empty or invalid.
    Stubbed,
}

/// A trait for detecting if a font has a DSIG table. This is useful when you
/// want to check if a font has a DSIG table without actually reading the
/// entire font data into memory.
pub trait FontDSIGDetector {
    /// The error type for detecting the DSIG table.
    type Error;
    /// Detects if the font has a DSIG table.
    fn check_for_dsig(&mut self) -> Result<DSIGType, Self::Error>;
}
