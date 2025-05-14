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

//! Various utilities for reading and writing WOFF1 font data for benchmarks.

use std::{
    io::{Read, Seek, Write},
    sync::OnceLock,
};

use c2pa_font_handler::{
    woff1::{
        directory::{Woff1Directory, Woff1DirectoryEntry},
        font::Woff1Font,
        header::Woff1Header,
    },
    FontDataExactRead, FontDataRead, MutFontDataWrite,
};

/// The number of entries in the WOFF1 font directory.
///
/// # Remarks
/// This is a constant value that represents the number of entries in the
/// WOFF1 font directory of the font loaded by `get_woff1_font_data()`.
pub const FONT_WOFF1_DIRECTORY_ENTRIES: usize = 10;

/// Static lock around the WOFF1 font data.
pub static WOFF1_FONT_DATA: OnceLock<Vec<u8>> = OnceLock::new();

/// Static lock around the WOFF1 font directory.
pub static WOFF1_DIRECTORY: OnceLock<Woff1Directory> = OnceLock::new();

/// Static lock around the WOFF1 font header.
pub static WOFF1_HEADER: OnceLock<Woff1Header> = OnceLock::new();

/// Gets the WOFF1 font data from the file system.
pub fn get_woff1_font_data() -> &'static [u8] {
    WOFF1_FONT_DATA
        .get_or_init(|| include_bytes!("../../../.devtools/font.woff").to_vec())
}

/// Gets the WOFF1 font header data from the file system.
pub fn get_woff1_header_data() -> &'static [u8] {
    &get_woff1_font_data()[..size_of::<Woff1Header>()]
}

/// Gets the WOFF1 font header from the file system.
pub fn get_woff1_header() -> &'static Woff1Header {
    WOFF1_HEADER.get_or_init(|| {
        let mut font_stream = std::io::Cursor::new(get_woff1_header_data());
        Woff1Header::from_reader(&mut font_stream)
            .expect("Failed to read font header")
    })
}

/// Gets the WOFF1 font directory data from the file system.
pub fn get_woff1_directory_data() -> &'static [u8] {
    &get_woff1_font_data()[size_of::<Woff1Header>()..]
}

/// Gets the WOFF1 font directory from the file system.
pub fn get_woff1_directory() -> &'static Woff1Directory {
    WOFF1_DIRECTORY.get_or_init(|| {
        let mut font_stream = std::io::Cursor::new(get_woff1_directory_data());
        Woff1Directory::from_reader_exact(
            &mut font_stream,
            0,
            size_of::<Woff1DirectoryEntry>() * FONT_WOFF1_DIRECTORY_ENTRIES,
        )
        .expect("Failed to read font directory")
    })
}

/// Parses the data in the font stream as a [`Woff1Font`] and executes the
/// provided function with the parsed font data object.
#[allow(dead_code)] // Remove this when C2PAUpdate has been implemented for Woff1Font
pub fn parse_font_and_execute<F, S>(font_stream: &mut S, execute: F)
where
    S: Read + Seek + ?Sized,
    F: FnOnce(Woff1Font),
{
    execute(
        Woff1Font::from_reader(font_stream).expect("Failed to read font data"),
    );
}

/// Parses the data in the font stream as a [`Woff1Font`] and executes the
/// provided function with the parsed font data object, then writes the modified
/// font data to the destination stream if provided.
#[allow(dead_code)] // Remove this when C2PAUpdate has been implemented for Woff1Font
pub fn parse_font_and_execute_write<F, S, W>(
    font_stream: &mut S,
    dest: Option<&mut W>,
    execute: F,
) where
    S: Read + Seek + ?Sized,
    W: Write + ?Sized,
    F: FnOnce(&mut Woff1Font),
{
    let mut font =
        Woff1Font::from_reader(font_stream).expect("Failed to read font data");
    execute(&mut font);
    if let Some(dest) = dest {
        font.write(dest).expect("Failed to write font data");
    }
}
