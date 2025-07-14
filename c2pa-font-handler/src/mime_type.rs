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
        let mut buffer = [0; 8];
        let mut reader = self.take(8);
        reader.read_exact(&mut buffer)?;

        // Rewind the reader to the original position
        self.seek(std::io::SeekFrom::Start(current_position))?;

        // Check for common font file signatures
        if buffer.starts_with(b"\x00\x01\x00\x00")
            || buffer.starts_with(b"\x4F\x54\x54\x4F")
        {
            Ok(MimeTypes::OTF.to_string())
        } else if buffer.starts_with(b"\x00\x01\x00\x00") {
            Ok(MimeTypes::TTF.to_string())
        } else if buffer.starts_with(b"\x77\x4F\x46\x46") {
            Ok(MimeTypes::WOFF.to_string())
        } else if buffer.starts_with(b"\x77\x4F\x46\x46\x32") {
            Ok(MimeTypes::WOFF2.to_string())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unknown font file format",
            ))
        }
    }
}
