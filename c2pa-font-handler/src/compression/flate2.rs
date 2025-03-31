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

//! Compression implementations using the flate2 library.
//!
//! The following example shows how to use the compression support:
//!
//! ```rust
//! use std::io::Cursor;
//!
//! use c2pa_font_handler::compression::{
//!     Compressor, Decompressor, ZlibCompression,
//! };
//!
//! // Compress the data
//! let data = b"Hello, world!";
//! // Create a new ZlibCompressor
//! let zlib_compression = ZlibCompression::default();
//! let result = zlib_compression.compress(data, Cursor::new(Vec::new()));
//! assert!(result.is_ok());
//! // Get the inner Vec<u8> to use as a compressed data
//! let compressed_data = result.unwrap().into_inner();
//! // Decompress the data
//! let result =
//!     zlib_compression.decompress(compressed_data, Cursor::new(Vec::new()));
//! assert!(result.is_ok());
//! let original = result.unwrap();
//! // The original data should be equal to the decompressed data
//! assert_eq!(data, original.get_ref().as_slice());
//! ```

use std::io::{Read, Write};

use super::{CompressionError, Compressor, Decoder, Decompressor, Encoder};

// Implementation of the Encoder trait for flate2::write::ZlibEncoder
impl<T: Write + Read> Encoder for flate2::write::ZlibEncoder<T> {
    type Error = CompressionError;
    type Stream = T;
}

// Implementation of the Decoder trait for flate2::write::ZlibDecoder
impl<T: Write + Read> Decoder for flate2::write::ZlibDecoder<T> {
    type Error = CompressionError;
    type Stream = T;
}

/// A compressor using the flate2 library.
#[derive(Default)]
pub struct ZlibCompression<T>
where
    T: Write + Read,
{
    stream_marker: std::marker::PhantomData<T>,
    compression_settings: flate2::Compression,
}

impl<T> ZlibCompression<T>
where
    T: Write + Read,
{
    /// Creates a new ZlibCompressor with the specified compression settings.
    pub fn new(compression_settings: flate2::Compression) -> Self {
        ZlibCompression {
            stream_marker: Default::default(),
            compression_settings,
        }
    }
}

impl<T: Write + Read> Compressor for ZlibCompression<T> {
    type Encoder = flate2::write::ZlibEncoder<T>;
    type Error = CompressionError;
    type Stream = T;

    fn compress<D: AsRef<[u8]>>(
        &self,
        data: D,
        destination: Self::Stream,
    ) -> Result<Self::Stream, Self::Error> {
        let mut encoder =
            Self::Encoder::new(destination, self.compression_settings);
        encoder.write_all(data.as_ref())?;
        let compressed_data = encoder.finish()?;
        Ok(compressed_data)
    }
}

impl<T: Write + Read> Decompressor for ZlibCompression<T> {
    type Decoder = flate2::write::ZlibDecoder<T>;
    type Error = CompressionError;
    type Stream = T;

    fn decompress<D: AsMut<[u8]>>(
        &self,
        data: D,
        destination: Self::Stream,
    ) -> Result<Self::Stream, Self::Error> {
        let mut decoder = Self::Decoder::new(destination);
        let mut data = data;
        decoder.write_all(data.as_mut())?;
        let decompressed_data = decoder.finish()?;
        Ok(decompressed_data)
    }
}

#[cfg(test)]
#[path = "flate2_test.rs"]
mod tests;
