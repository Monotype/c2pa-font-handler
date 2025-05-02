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

use super::*;

#[test]
fn round_trip_compress_decompress() {
    let data = b"Hello, world!";
    let mut compressed_data = Vec::new();
    {
        let mut compressor = CompressingWriter::new(&mut compressed_data);
        compressor.write_all(data).unwrap();
        compressor.flush().unwrap();
    }

    let mut compressed_data_cursor = Cursor::new(&compressed_data);
    let mut decompressor =
        DecompressingReader::new(&mut compressed_data_cursor);
    let mut decompressed_data = Vec::new();
    decompressor.read_to_end(&mut decompressed_data).unwrap();

    assert_eq!(data, decompressed_data.as_slice());
}

#[test]
fn round_trip_custom_compression_level() {
    let data = b"Hello, world!";
    let compression_level = Compression::new(6); // Custom compression level
    let mut compressed_data = Vec::new();
    {
        let mut compressor = CompressingWriter::with_compression(
            &mut compressed_data,
            compression_level,
        );
        compressor.write_all(data).unwrap();
        let _ = compressor.finish().unwrap();
    }

    let mut compressed_data_cursor = Cursor::new(&compressed_data);
    let mut decompressor =
        DecompressingReader::new(&mut compressed_data_cursor);
    let mut decompressed_data = Vec::new();
    decompressor.read_to_end(&mut decompressed_data).unwrap();

    assert_eq!(data, decompressed_data.as_slice());
}
