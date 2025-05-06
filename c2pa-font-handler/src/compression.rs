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

#[cfg(feature = "flate")]
mod flate2;
#[cfg(feature = "flate")]
pub use flate2::*;

/// Errors related to compression.
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    /// A compression error with the flate2 library.
    #[cfg(feature = "flate")]
    #[error(transparent)]
    Flate2CompressError(#[from] ::flate2::CompressError),
    /// A decompression error with the flate2 library.
    #[cfg(feature = "flate")]
    #[error(transparent)]
    Flate2DecompressError(#[from] ::flate2::DecompressError),
    /// General compression error.
    #[error("General compression error: {0}")]
    General(String),
    /// An error occurred while reading or writing data.
    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
}
