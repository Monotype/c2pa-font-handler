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

//! Help with MIME types for font files.

use std::io::{Read, Seek};

/// MIME types for common font formats.
pub struct MimeTypes {}

impl MimeTypes {
    /// MIME type for OpenType font files.
    pub const OTF: &str = "font/otf";
    /// MIME type for TrueType font files.
    pub const TTF: &str = "font/ttf";
    /// MIME type for WOFF font files.
    pub const WOFF: &str = "font/woff";
    /// MIME type for WOFF2 font files.
    pub const WOFF2: &str = "font/woff2";
}

/// Magic numbers for font file formats.
struct MagicNumbers {}

impl MagicNumbers {
    /// Magic number for OpenType font files with PostScript outlines.
    const CFF_OTF: [u8; 4] = *b"OTTO";
    /// Magic number for TrueType font files.
    const TTF: [u8; 4] = *b"true";
    /// Magic number for TrueType-based OpenType font files.
    const TTF_OTF: [u8; 4] = [0x00, 0x01, 0x00, 0x00];
    /// Magic number for TrueType font files.
    const WOFF: [u8; 4] = *b"wOFF";
    /// Magic number for WOFF2 font files.
    const WOFF2: [u8; 4] = *b"wOF2";
}

/// A way to guess the MIME type from an object.
pub trait MimeTypeGuesser {
    /// The error type for the MIME type guessing.
    type Error;

    /// Guess the MIME type of the file based on its contents.
    fn guess_mime_type(&mut self) -> Result<String, Self::Error>;
}

impl<T: Read + Seek + ?Sized> MimeTypeGuesser for T {
    type Error = std::io::Error;

    fn guess_mime_type(&mut self) -> Result<String, Self::Error> {
        // Grab the current position so we can rewind later
        let current_position = self.stream_position()?;
        // Read the first few bytes to determine the MIME type
        let mut buffer = [0; 4];
        let mut reader = self.take(4);
        reader.read_exact(&mut buffer)?;

        // Rewind the reader to the original position
        self.seek(std::io::SeekFrom::Start(current_position))?;

        // Check for common font file signatures
        if buffer.starts_with(&MagicNumbers::TTF_OTF)
            || buffer.starts_with(&MagicNumbers::CFF_OTF)
        {
            Ok(MimeTypes::OTF.to_string())
        } else if buffer.starts_with(&MagicNumbers::TTF) {
            Ok(MimeTypes::TTF.to_string())
        } else if buffer.starts_with(&MagicNumbers::WOFF) {
            Ok(MimeTypes::WOFF.to_string())
        } else if buffer.starts_with(&MagicNumbers::WOFF2) {
            Ok(MimeTypes::WOFF2.to_string())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unknown font file format",
            ))
        }
    }
}

#[cfg(test)]
#[path = "mime_type_test.rs"]
mod tests;
