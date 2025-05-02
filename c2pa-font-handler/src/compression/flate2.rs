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

//! Decompression/Compression support using the flate2 library.
//!
//! The following example shows how to use the wrapper streams to compress
//! and decompress data.
//!
//! ```rust
//! use std::io::{Cursor, Read, Write};
//!
//! use c2pa_font_handler::compression::{
//!     CompressingWriter, CompressionError, DecompressingReader,
//! };
//!
//! # fn main() -> Result<(), CompressionError> {
//! // Data to compress
//! let data = b"Hello, world!";
//!
//! // Compress the data
//! let mut compressed_data = Vec::new();
//! {
//!     // Create the `CompressingWriter` and write the data to it
//!     let mut compressor = CompressingWriter::new(&mut compressed_data);
//!     compressor.write_all(data)?;
//!     compressor.finish()?;
//! }
//!
//! // Create a cursor for the compressed data, for reading
//! let mut compressed_data_cursor = Cursor::new(&compressed_data);
//! // Create the `DecompressingReader`
//! let mut decompressor =
//!     DecompressingReader::new(&mut compressed_data_cursor);
//! // And create a buffer to hold the decompressed data
//! let mut decompressed_data = Vec::new();
//! // Read the decompressed data into the buffer
//! decompressor.read_to_end(&mut decompressed_data).unwrap();
//!
//! assert_eq!(data, decompressed_data.as_slice());
//! # Ok::<(), CompressionError>(())
//! # }
//! ```

use std::io::{Read, Write};

use flate2::{write::GzEncoder, Compression};

use super::CompressionError;

/// A structure for writing bytes to which compression is applied.
pub struct CompressingWriter<'a, S: 'a + Write> {
    encoder: GzEncoder<&'a mut S>,
}

impl<'a, S: 'a + Write> CompressingWriter<'a, S> {
    /// Creates a new [`CompressingWriter`] with the default compression level.
    pub fn new(inner: &'a mut S) -> Self {
        Self {
            encoder: GzEncoder::new(inner, Compression::default()),
        }
    }

    /// Creates a new [`CompressingWriter`] with the specified compression
    /// level.
    pub fn with_compression(
        inner: &'a mut S,
        compression: Compression,
    ) -> Self {
        Self {
            encoder: GzEncoder::new(inner, compression),
        }
    }

    /// Finishes the compression and returns the underlying stream.
    pub fn finish(self) -> Result<&'a mut S, CompressionError> {
        let stream = self.encoder.finish()?;
        Ok(stream)
    }
}

// Implement `Write` for `CompressingWriter`, allowing it to be used as a
// standard writer.
impl<'a, S: 'a + Write> Write for CompressingWriter<'a, S> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.encoder.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.encoder.flush()
    }
}

/// A structure for reading bytes from a compressed stream.
pub struct DecompressingReader<'a, S: 'a + Read> {
    // The underlying decoder used for decompression.
    decoder: flate2::read::GzDecoder<&'a mut S>,
}

impl<'a, S: 'a + Read> DecompressingReader<'a, S> {
    /// Creates a new [`DecompressingReader`].
    pub fn new(inner: &'a mut S) -> Self {
        Self {
            decoder: flate2::read::GzDecoder::new(inner),
        }
    }
}

// Implement `Read` for `DecompressingReader`, allowing it to be used as a
// standard reader.
impl<'a, S: 'a + Read> Read for DecompressingReader<'a, S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.decoder.read(buf)
    }
}

#[cfg(test)]
#[path = "flate2_test.rs"]
mod tests;
