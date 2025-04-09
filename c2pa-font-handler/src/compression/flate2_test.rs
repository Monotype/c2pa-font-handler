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

//! Tests for the flate2 compression support

use std::io::Cursor;

use flate2::Compression;

use super::*;

#[test]
fn compress() {
    let data = b"Hello, world!";
    // And use it to build a flate2 compressor
    let zlib_compression = ZlibCompression::default();

    // Create a destination to compress to
    let destination = Cursor::new(Vec::new());
    // And compress the data
    let result = zlib_compression.compress(data, destination);
    assert!(result.is_ok());
    let destination = result.unwrap().into_inner();

    // We will setup to decode for a round trip test
    let original = Cursor::new(Vec::new());
    let result = zlib_compression.decompress(destination, original);
    assert!(result.is_ok());
    let original = result.unwrap();
    assert_eq!(data, original.get_ref().as_slice());
}

#[test]
fn compression_with_custom_settings() {
    let data = b"Hello, world!";
    // Create a new ZlibCompressor with custom settings
    let zlib_compression = ZlibCompression::new(Compression::fast());
    // Create a destination to compress to
    let destination = Cursor::new(Vec::new());
    // And compress the data
    let result = zlib_compression.compress(data, destination);
    assert!(result.is_ok());
    let destination = result.unwrap().into_inner();

    // We will setup to decode for a round trip test
    let original = Cursor::new(Vec::new());
    let result = zlib_compression.decompress(destination, original);
    assert!(result.is_ok());
    let original = result.unwrap();
    assert_eq!(data, original.get_ref().as_slice());
}
