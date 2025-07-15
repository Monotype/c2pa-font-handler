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

/// A very slim representation of MIME types for font files.
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

/// Various known magic types with their MIME types.
pub struct MagicTypes {
    /// The magic number for the font file format.
    magic: &'static [u8; 4],
    /// The MIME type associated with the magic number.
    mime_type: &'static str,
}

impl MagicTypes {
    /// Known MIME types for font files.
    pub const KNOWN_TYPES: &'static [MagicTypes] =
        &[Self::OTF, Self::TTF, Self::TTF_OTF, Self::WOFF, Self::WOFF2];
    /// OpenType font magic number and MIME type.
    pub const OTF: MagicTypes = MagicTypes {
        magic: b"OTTO",
        mime_type: MimeTypes::OTF,
    };
    /// TrueType font magic number and MIME type.
    pub const TTF: MagicTypes = MagicTypes {
        magic: b"true",
        mime_type: MimeTypes::TTF,
    };
    /// TrueType-based OpenType font magic number and MIME type.
    pub const TTF_OTF: MagicTypes = MagicTypes {
        magic: &[0x00, 0x01, 0x00, 0x00],
        mime_type: MimeTypes::OTF,
    };
    /// WOFF font magic number and MIME type.
    pub const WOFF: MagicTypes = MagicTypes {
        magic: b"wOFF",
        mime_type: MimeTypes::WOFF,
    };
    /// WOFF2 font magic number and MIME type.
    pub const WOFF2: MagicTypes = MagicTypes {
        magic: b"wOF2",
        mime_type: MimeTypes::WOFF2,
    };
}

/// Error type for MIME type guessing.
#[derive(Debug, thiserror::Error)]
pub enum MimeTypeError {
    /// An error while reading/writing font data.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    /// Error when the magic number does not match any known type.
    #[error("Unknown font file format")]
    UnknownMagicType,
}

/// A way to guess the MIME type from an object.
pub trait FontMimeTypeGuesser {
    /// The error type for the MIME type guessing.
    type Error;

    /// Guess the MIME type of the file based on its contents.
    fn guess_mime_type(&mut self) -> Result<&'static str, Self::Error>;
}

impl<T: Read + Seek + ?Sized> FontMimeTypeGuesser for T {
    type Error = MimeTypeError;

    fn guess_mime_type(&mut self) -> Result<&'static str, Self::Error> {
        // Grab the current position so we can rewind later
        let current_position = self.stream_position()?;
        // Read the first few bytes to determine the MIME type
        let mut buffer = [0; 4];
        let mut reader = self.take(4);
        reader.read_exact(&mut buffer)?;

        // Rewind the reader to the original position
        self.seek(std::io::SeekFrom::Start(current_position))?;

        MagicTypes::KNOWN_TYPES
            .iter()
            .find(|&magic_type| buffer.starts_with(magic_type.magic))
            .map(|magic_type| magic_type.mime_type)
            .ok_or(MimeTypeError::UnknownMagicType)
    }
}

#[cfg(test)]
#[path = "mime_type_test.rs"]
mod tests;
