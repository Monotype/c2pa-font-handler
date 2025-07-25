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

use std::{
    fmt::Display,
    io::{Read, Seek},
};

use byteorder::{BigEndian, ReadBytesExt};

use crate::magic::Magic;

/// A very slim representation of MIME types for font files.
#[derive(Debug, PartialEq, Eq)]
pub enum FontMimeTypes {
    /// OpenType font MIME type.
    OTF,
    /// TrueType font MIME type.
    TTF,
    /// WOFF font MIME type.
    WOFF,
    /// WOFF2 font MIME type.
    WOFF2,
}

impl Display for FontMimeTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontMimeTypes::OTF => write!(f, "font/otf"),
            FontMimeTypes::TTF => write!(f, "font/ttf"),
            FontMimeTypes::WOFF => write!(f, "font/woff"),
            FontMimeTypes::WOFF2 => write!(f, "font/woff2"),
        }
    }
}

/// Various known magic types with their MIME types.
pub struct MagicTypes {
    /// The magic number for the font file format.
    magic: Magic,
    /// The MIME type associated with the magic number.
    mime_type: FontMimeTypes,
}

impl MagicTypes {
    /// Known MIME types for font files.
    pub const KNOWN_TYPES: &'static [MagicTypes] =
        &[Self::OTF, Self::TTF, Self::TTF_OTF, Self::WOFF, Self::WOFF2];
    /// OpenType font magic number and MIME type.
    pub const OTF: MagicTypes = MagicTypes {
        magic: Magic::OpenType,
        mime_type: FontMimeTypes::OTF,
    };
    /// TrueType MIME type.
    pub const TTF: MagicTypes = MagicTypes {
        magic: Magic::TrueType,
        mime_type: FontMimeTypes::TTF,
    };
    /// Apple True OpenType font magic number and MIME type.
    pub const TTF_OTF: MagicTypes = MagicTypes {
        magic: Magic::AppleTrue,
        mime_type: FontMimeTypes::OTF,
    };
    /// WOFF font magic number and MIME type.
    pub const WOFF: MagicTypes = MagicTypes {
        magic: Magic::Woff,
        mime_type: FontMimeTypes::WOFF,
    };
    /// WOFF2 font magic number and MIME type.
    pub const WOFF2: MagicTypes = MagicTypes {
        magic: Magic::Woff2,
        mime_type: FontMimeTypes::WOFF2,
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
    fn guess_mime_type(
        &mut self,
    ) -> Result<&'static FontMimeTypes, Self::Error>;
}

impl<T: Read + Seek + ?Sized> FontMimeTypeGuesser for T {
    type Error = MimeTypeError;

    fn guess_mime_type(
        &mut self,
    ) -> Result<&'static FontMimeTypes, Self::Error> {
        // Grab the current position so we can rewind later
        let current_position = self.stream_position()?;
        // Read the first few bytes to determine the MIME type
        let mut reader = self.take(4);
        let magic = reader.read_u32::<BigEndian>()?;

        // Rewind the reader to the original position
        self.seek(std::io::SeekFrom::Start(current_position))?;

        MagicTypes::KNOWN_TYPES
            .iter()
            .find(|&magic_type| magic == magic_type.magic as u32)
            .map(|magic_type| &magic_type.mime_type)
            .ok_or(MimeTypeError::UnknownMagicType)
    }
}

#[cfg(test)]
#[path = "mime_type_test.rs"]
mod tests;
