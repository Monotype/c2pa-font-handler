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

//! Compression support for fonts (mostly WOFF1/WOFF2).
//!
//! Currently the only supported compression is Zlib.

use std::io::{Read, Seek, Write};
#[cfg(feature = "flate")]
mod flate2;
#[cfg(feature = "flate")]
pub use flate2::ZlibCompression;

/// Errors related to compression.
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    /// A compression error with the flate2 library.
    #[error(transparent)]
    Flate2CompressError(#[from] ::flate2::CompressError),
    /// A decompression error with the flate2 library.
    #[error(transparent)]
    Flate2DecompressError(#[from] ::flate2::DecompressError),
    /// General compression error.
    #[error("General compression error: {0}")]
    General(String),
    /// An error occurred while reading or writing data.
    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
}

/// A trait representing a data compressor.
pub trait Compressor {
    /// The error type returned by the compressor.
    type Error: Into<CompressionError>;

    /// The type of writer used by the compressor.
    type Stream: Write + Read + Seek;

    /// The type of encoder used by the compressor.
    type Encoder: Encoder<Error = Self::Error, Stream = Self::Stream>;

    /// Compresses the given data.
    fn compress<T: AsRef<[u8]>>(
        &self,
        data: T,
        destination: Self::Stream,
    ) -> Result<Self::Stream, Self::Error>;
}

/// A trait representing a data decompressor.
pub trait Decompressor {
    /// The error type returned by the decompressor.
    type Error: Into<CompressionError>;

    /// The type of writer used by the decompressor.
    type Stream: Write + Read + Seek;

    /// The type of decoder used by the decompressor.
    type Decoder: Decoder<Error = Self::Error, Stream = Self::Stream>;

    /// Decompresses the given data.
    fn decompress<T: AsMut<[u8]>>(
        &self,
        data: T,
        destination: Self::Stream,
    ) -> Result<Self::Stream, Self::Error>;
}

/// Describes an encoder to use during compression.
pub trait Encoder: Write {
    /// The error type returned by the encoder.
    type Error: Into<CompressionError>;

    /// The type of writer used by the encoder.
    type Stream: Write + Read + Seek;
}

/// Describes a decoder to use during decompression.
pub trait Decoder: Write + Read {
    /// The error type returned by the decoder.
    type Error: Into<CompressionError>;

    /// The type of writer used by the decoder.
    type Stream: Write + Read + Seek;
}
