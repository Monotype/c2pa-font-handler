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

//! WOFF1 font format

use crate::{data::Data, FontDirectoryEntry};

pub mod directory;
pub mod font;
pub mod header;

/// WOFF1 table
pub type Table = Data;
/// Extension of the [`FontDirectoryEntry`] trait for WOFF1 tables, which adds a
/// method to check if the table is compressed.
pub trait WoffDirectoryEntry: FontDirectoryEntry {
    /// Checks if the table is compressed
    fn is_compressed(&self) -> bool;
}

/// Provides access to the `metadata` section of the WOFF file.
pub trait WoffMetadata {
    /// Returns the uncompressed metadata section, if any, of the WOFF file.
    fn metadata(&self) -> Option<Vec<u8>>;
}
