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

//! Various utilities for reading and writing SFNT font data for benchmarks.

use std::{
    io::{Read, Seek, Write},
    sync::OnceLock,
};

use c2pa_font_handler::{
    sfnt::{
        directory::{SfntDirectory, SfntDirectoryEntry},
        font::SfntFont,
        header::SfntHeader,
    },
    FontDataExactRead, FontDataRead, MutFontDataWrite,
};

/// The number of entries in the SFNT font directory.
///
/// # Remarks
/// This is a constant value that represents the number of entries in the
/// SFNT font directory of the font loaded by `get_sfnt_font_data()`.
pub const FONT_OTF_DIRECTORY_ENTRIES: usize = 11;

/// Static lock around the SFNT font data.
pub static SFNT_FONT_DATA: OnceLock<Vec<u8>> = OnceLock::new();

/// Static lock around the SFNT font directory.
pub static SFNT_DIRECTORY: OnceLock<SfntDirectory> = OnceLock::new();

/// Static lock around the SFNT font header.
pub static SFNT_HEADER: OnceLock<SfntHeader> = OnceLock::new();

/// Gets the SFNT font data from the file system.
pub fn get_sfnt_font_data() -> &'static [u8] {
    SFNT_FONT_DATA
        .get_or_init(|| include_bytes!("../../../.devtools/font.otf").to_vec())
}

/// Gets the SFNT font header data from the file system.
pub fn get_sfnt_header_data() -> &'static [u8] {
    &get_sfnt_font_data()[..size_of::<SfntHeader>()]
}

/// Gets the SFNT font header from the file system.
pub fn get_sfnt_header() -> &'static SfntHeader {
    SFNT_HEADER.get_or_init(|| {
        let mut font_stream = std::io::Cursor::new(get_sfnt_header_data());
        SfntHeader::from_reader(&mut font_stream)
            .expect("Failed to read font header")
    })
}

/// Gets the SFNT font directory data from the file system.
pub fn get_sfnt_directory_data() -> &'static [u8] {
    &get_sfnt_font_data()[size_of::<SfntHeader>()..]
}

/// Gets the SFNT font directory from the file system.
pub fn get_sfnt_directory() -> &'static SfntDirectory {
    SFNT_DIRECTORY.get_or_init(|| {
        let mut font_stream = std::io::Cursor::new(get_sfnt_directory_data());
        SfntDirectory::from_reader_exact(
            &mut font_stream,
            0,
            size_of::<SfntDirectoryEntry>() * FONT_OTF_DIRECTORY_ENTRIES,
        )
        .expect("Failed to read font directory")
    })
}

/// Parses the data in the font stream as a [`SfntFont`] and executes the
/// provided function with the parsed font data object.
pub fn parse_font_and_execute<F, S>(font_stream: &mut S, execute: F)
where
    S: Read + Seek + ?Sized,
    F: FnOnce(SfntFont),
{
    execute(
        SfntFont::from_reader(font_stream).expect("Failed to read font data"),
    );
}

/// Parses the data in the font stream as a [`SfntFont`] and executes the
/// provided function with the parsed font data object, then writes the modified
/// font data to the destination stream if provided.
pub fn parse_font_and_execute_write<F, S, W>(
    font_stream: &mut S,
    dest: Option<&mut W>,
    execute: F,
) where
    S: Read + Seek + ?Sized,
    W: Write + ?Sized,
    F: FnOnce(&mut SfntFont),
{
    let mut font =
        SfntFont::from_reader(font_stream).expect("Failed to read font data");
    execute(&mut font);
    if let Some(dest) = dest {
        font.write(dest).expect("Failed to write font data");
    }
}
