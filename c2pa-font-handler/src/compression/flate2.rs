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
//!     let mut compressor =
//!         CompressingWriter::builder(&mut compressed_data).build();
//!     compressor.write_all(data)?;
//!     compressor.finish()?;
//! }
//!
//! // Create a cursor for the compressed data, for reading
//! let mut compressed_data_cursor = Cursor::new(&compressed_data);
//! // Create the `DecompressingReader`
//! let mut decompressor =
//!     DecompressingReader::builder(&mut compressed_data_cursor).build();
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

use flate2::Compression;

use super::CompressionError;

/// The available compression algorithms.
pub enum EncoderDecoderAlgorithm {
    /// Zlib compression algorithm.
    Zlib,
}

/// The available compression encoders.
pub enum Encoders<'a, S: 'a + Write + ?Sized> {
    /// Zlib encoder.
    Zlib(flate2::write::ZlibEncoder<&'a mut S>),
    // TODO: Add more decoders as needed (i.e., when we go to support WOFF2).
}

/// The available decompression decoders.
pub enum Decoders<'a, S: 'a + Read + ?Sized> {
    /// Zlib decoder.
    Zlib(flate2::read::ZlibDecoder<&'a mut S>),
    // TODO: Add more decoders as needed (i.e., when we go to support WOFF2).
}

impl<'a, S: 'a + Write + ?Sized> Encoders<'a, S> {
    /// Creates a new encoder with the specified algorithm and compression
    /// level.
    pub fn with_compression(
        inner: &'a mut S,
        algorithm: EncoderDecoderAlgorithm,
        compression: Compression,
    ) -> Self {
        match algorithm {
            EncoderDecoderAlgorithm::Zlib => Encoders::Zlib(
                flate2::write::ZlibEncoder::new(inner, compression),
            ),
        }
    }

    /// Finishes the compression and returns the underlying stream.
    pub fn finish(self) -> Result<&'a mut S, std::io::Error> {
        match self {
            Encoders::Zlib(encoder) => encoder.finish(),
        }
    }

    /// Writes data to the encoder, applying compression.
    pub fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Encoders::Zlib(encoder) => encoder.write(buf),
        }
    }

    /// Flushes the encoder, ensuring all data is written to the underlying
    /// stream.
    pub fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Encoders::Zlib(encoder) => encoder.flush(),
        }
    }
}

impl<'a, S: 'a + Read + ?Sized> Decoders<'a, S> {
    /// Creates a new decoder with the specified algorithm.
    pub fn new(inner: &'a mut S, algorithm: EncoderDecoderAlgorithm) -> Self {
        match algorithm {
            EncoderDecoderAlgorithm::Zlib => {
                Decoders::Zlib(flate2::read::ZlibDecoder::new(inner))
            }
        }
    }

    /// Reads data from the decoder, decompressing it into the provided buffer.
    pub fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Decoders::Zlib(decoder) => decoder.read(buf),
        }
    }
}

/// A structure for writing bytes to which compression is applied.
pub struct CompressingWriter<'a, S: 'a + Write + ?Sized> {
    encoder: Encoders<'a, S>,
}

impl<'a, S: 'a + Write + ?Sized> CompressingWriter<'a, S> {
    /// Creates a new [`CompressingWriter`] with the default compression level.
    fn new(encoder: Encoders<'a, S>) -> Self {
        Self { encoder }
    }

    /// Creates a new [`CompressingWriterBuilder`] for the given stream.
    pub fn builder(inner: &'a mut S) -> CompressingWriterBuilder<'a, S> {
        CompressingWriterBuilder::new(inner)
    }

    /// Finishes the compression and returns the underlying stream.
    pub fn finish(self) -> Result<&'a mut S, CompressionError> {
        let stream = self.encoder.finish()?;
        Ok(stream)
    }
}

// Implement `Write` for `CompressingWriter`, allowing it to be used as a
// standard writer.
impl<'a, S: 'a + Write + ?Sized> Write for CompressingWriter<'a, S> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.encoder.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.encoder.flush()
    }
}

/// A builder for creating a [`CompressingWriter`].
pub struct CompressingWriterBuilder<'a, S: 'a + Write + ?Sized> {
    inner: &'a mut S,
    compression: Compression,
    algorithm: EncoderDecoderAlgorithm,
}

impl<'a, S: 'a + Write + ?Sized> CompressingWriterBuilder<'a, S> {
    /// Creates a new [`CompressingWriterBuilder`] with the specified
    /// compression level and algorithm.
    pub fn new(inner: &'a mut S) -> Self {
        Self {
            inner,
            compression: Compression::default(),
            algorithm: EncoderDecoderAlgorithm::Zlib,
        }
    }

    /// Sets the compression level.
    pub fn with_compression(self, compression: Compression) -> Self {
        Self {
            compression,
            ..self
        }
    }

    /// Sets the compression algorithm.
    pub fn with_algorithm(self, algorithm: EncoderDecoderAlgorithm) -> Self {
        Self { algorithm, ..self }
    }

    /// Builds the [`CompressingWriter`].
    pub fn build(self) -> CompressingWriter<'a, S> {
        let encoder = Encoders::with_compression(
            self.inner,
            self.algorithm,
            self.compression,
        );
        CompressingWriter::new(encoder)
    }
}

/// A structure for reading bytes from a compressed stream.
pub struct DecompressingReader<'a, S: 'a + Read + ?Sized> {
    // The underlying decoder used for decompression.
    decoder: Decoders<'a, S>,
}

impl<'a, S: 'a + Read + ?Sized> DecompressingReader<'a, S> {
    /// Creates a new [`DecompressingReader`].
    pub fn new(decoder: Decoders<'a, S>) -> Self {
        Self { decoder }
    }

    /// Creates a new [`DecompressingReaderBuilder`] for the given stream.
    pub fn builder(inner: &'a mut S) -> DecompressingReaderBuilder<'a, S> {
        DecompressingReaderBuilder::new(inner)
    }
}

// Implement `Read` for `DecompressingReader`, allowing it to be used as a
// standard reader.
impl<'a, S: 'a + Read + ?Sized> Read for DecompressingReader<'a, S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.decoder.read(buf)
    }
}

/// A builder for creating a [`DecompressingReader`] types.
pub struct DecompressingReaderBuilder<'a, S: 'a + Read + ?Sized> {
    /// The underlying stream to read from.
    inner: &'a mut S,
    /// The decoder algorithm to use.
    algorithm: EncoderDecoderAlgorithm,
}

impl<'a, S: 'a + Read + ?Sized> DecompressingReaderBuilder<'a, S> {
    /// Creates a new [`DecompressingReaderBuilder`] with the specified
    /// algorithm.
    pub fn new(inner: &'a mut S) -> Self {
        Self {
            inner,
            algorithm: EncoderDecoderAlgorithm::Zlib,
        }
    }

    /// Sets the compression algorithm.
    pub fn with_algorithm(self, algorithm: EncoderDecoderAlgorithm) -> Self {
        Self { algorithm, ..self }
    }

    /// Builds the [`DecompressingReader`].
    pub fn build(self) -> DecompressingReader<'a, S> {
        DecompressingReader::new(Decoders::new(self.inner, self.algorithm))
    }
}

#[cfg(test)]
#[path = "flate2_test.rs"]
mod tests;
