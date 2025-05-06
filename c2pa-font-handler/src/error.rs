// Copyright 2024-2025 Monotype Imaging Inc.
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

//! Errors related to font I/O.

use super::tag::FontTag;

/// Errors related to font I/O.
#[derive(Debug, thiserror::Error)]
pub enum FontIoError {
    /// An error occurred while compressing/decompressing the font data.
    #[cfg(feature = "compression")]
    #[error(transparent)]
    CompressionError(#[from] crate::compression::CompressionError),
    /// A content credential already exists
    #[error("A content credential already exists")]
    ContentCredentialAlreadyExists,
    /// Content credential record not found
    #[error("A content credential was not found")]
    ContentCredentialNotFound,
    /// Failed to write the font data.
    #[error("Failed to write font data")]
    FailedToWriteFontData(std::io::Error),
    /// Failed to write the font table data.
    #[error("Failed to write font table data")]
    FailedToWriteTableData(std::io::Error),
    /// An error occurred while reading or writing the font data.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    /// An invalid (or unsupported) major C2PA version
    #[error("Invalid major version specified for a valid C2PA record")]
    InvalidC2paMajorVersion(u16),
    /// An invalid (or unsupported) minor C2PA version
    #[error("Invalid minor version specified for a valid C2PA record")]
    InvalidC2paMinorVersion(u16),
    /// The magic number in the 'head' table is invalid.
    #[error("Invalid magic number in the 'head' table; expected 0x5f0f3cf5, got {0}")]
    InvalidHeadMagicNumber(u32),
    /// The specified size for reading a table directory entry record is
    /// invalid.
    #[error("Invalid size for a table directory entry record, expected {expected} bytes, got {got}")]
    InvalidSizeForDirectoryEntry {
        /// Expected size
        expected: usize,
        /// The actual size specified
        got: usize,
    },
    /// The specified size for reading a directory is not 4-byte aligned.
    #[error("Invalid size for a table directory entry record, expected a 4-byte aligned request, got {0}")]
    InvalidSizeForDirectory(usize),
    /// The specified size for reading a header is invalid.
    #[error("Invalid size for a header, expected 12 bytes, got {0}")]
    InvalidSizeForHeader(usize),
    /// The specified size for reading a tag is invalid.
    #[error("Invalid size for a tag, expected 4 bytes, got {0}")]
    InvalidSizeForTAG(usize),
    /// The font table is truncated.
    #[error("The font table is truncated: {0}")]
    LoadTableTruncated(FontTag),
    /// Save errors.
    #[error("Error saving the font: {0}")]
    SaveError(#[from] FontSaveError),
    /// An error occurred while generating a string from UTF-8 bytes.
    #[error("Error occurred while generating a string from UTF-8 bytes: {0}")]
    StringFromUtf8(#[from] std::string::FromUtf8Error),
    /// When determining the type of font, the magic number was not recognized.
    #[error("An unknown magic number was encountered: {0}")]
    UnknownMagic(u32),
}

/// Errors related to saving a font
#[derive(Debug, thiserror::Error)]
pub enum FontSaveError {
    /// The font has no tables.
    #[error("No tables were found in the font.")]
    NoTablesFound,
    /// The font has too many tables that were added.
    #[error("Too many tables were added to the font, which is currently not supported.")]
    TooManyTablesAdded,
    /// The font has too many tables that were removed.
    #[error("Too many tables were removed from the font, which is currently not supported.")]
    TooManyTablesRemoved,
    /// An unexpected table was encountered.
    #[error("An unexpected table was encountered: {0}")]
    UnexpectedTable(String),
}
