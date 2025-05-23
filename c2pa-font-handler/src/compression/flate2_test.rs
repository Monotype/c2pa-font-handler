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
        let mut compressor =
            CompressingWriter::builder(&mut compressed_data).build();
        compressor.write_all(data).unwrap();
        compressor.flush().unwrap();
    }

    let mut compressed_data_cursor = Cursor::new(&compressed_data);
    let mut decompressor =
        DecompressingReader::builder(&mut compressed_data_cursor)
            .with_algorithm(EncoderDecoderAlgorithm::Zlib)
            .build();
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
        let mut compressor = CompressingWriter::builder(&mut compressed_data)
            .with_compression(compression_level)
            .build();
        compressor.write_all(data).unwrap();
        let _ = compressor.finish().unwrap();
    }

    let mut compressed_data_cursor = Cursor::new(&compressed_data);
    let mut decompressor =
        DecompressingReader::builder(&mut compressed_data_cursor).build();
    let mut decompressed_data = Vec::new();
    decompressor.read_to_end(&mut decompressed_data).unwrap();

    assert_eq!(data, decompressed_data.as_slice());
}

#[test]
fn round_trip_compress_decompress_with_algorithm() {
    let data = b"Hello, world!";
    let mut compressed_data = Vec::new();
    {
        let mut compressor = CompressingWriter::builder(&mut compressed_data)
            .with_algorithm(EncoderDecoderAlgorithm::Zlib)
            .build();
        compressor.write_all(data).unwrap();
        compressor.flush().unwrap();
    }

    let mut compressed_data_cursor = Cursor::new(&compressed_data);
    let mut decompressor =
        DecompressingReader::builder(&mut compressed_data_cursor)
            .with_algorithm(EncoderDecoderAlgorithm::Zlib)
            .build();
    let mut decompressed_data = Vec::new();
    decompressor.read_to_end(&mut decompressed_data).unwrap();

    assert_eq!(data, decompressed_data.as_slice());
}

// Test for compressing and decompressing a portion of the stream.
// This is useful for testing the ability to handle partial reads/writes
// (for example just decompressing one table from a WOFF font file).
#[test]
fn round_trip_compress_decompress_portion_of_stream() {
    // Our control data
    let data = b"Hello, world!";

    // Create a buffer to hold the compressed data
    let mut compressed_data = Vec::new();
    let header = b"Header";
    // First put a pretend header
    let _ = compressed_data.write(header).unwrap(); // Simulate a header
                                                    // Then compress our control data
    {
        let mut compressor =
            CompressingWriter::builder(&mut compressed_data).build();
        compressor.write_all(data).unwrap();
        compressor.flush().unwrap();
    }
    // Get the compressed length
    let compressed_length = compressed_data.len() - header.len(); // Subtract the header length
                                                                  // And add a footer to the compressed data
    let _ = compressed_data.write(b"Footer").unwrap(); // Simulate a footer

    // Setup to decompress the data from the portion of the stream
    let mut decompressed_data = Vec::new();
    let mut compressed_data_cursor = Cursor::new(&compressed_data);
    // Set the position to the start of the compressed data
    {
        // Skip the header
        compressed_data_cursor.set_position(header.len() as u64);
        // Take a portion of the stream that is the length of the compressed
        // data
        let mut temp = compressed_data_cursor
            .by_ref()
            .take(compressed_length as u64);
        // Build the decompressor with the temporary stream
        let mut decompressor = DecompressingReader::builder(&mut temp).build();
        // Read to the end of the temporary limited stream to fill the buffer
        let bytes_read =
            decompressor.read_to_end(&mut decompressed_data).unwrap();
        // Check that we read the expected number of bytes
        assert_eq!(bytes_read, data.len());
    }
    println!("position: {}", compressed_data_cursor.position());
    assert_eq!(data, decompressed_data.as_slice());
    let mut footer_buf = Vec::new(); // Size of the footer
    compressed_data_cursor.read_to_end(&mut footer_buf).unwrap();
    assert_eq!(&footer_buf, b"Footer");
}

#[test]
fn test_decompressing_reader_read_n_bytes_success() {
    let data = b"Hello, world!";
    let mut compressed_data = Vec::new();
    {
        let mut compressor =
            CompressingWriter::builder(&mut compressed_data).build();
        compressor.write_all(data).unwrap();
        compressor.flush().unwrap();
    }

    let mut compressed_data_cursor = Cursor::new(&compressed_data);
    let mut decompressor =
        DecompressingReader::builder(&mut compressed_data_cursor).build();
    let mut buffer = vec![0u8; data.len()];
    decompressor.read_exact(&mut buffer).unwrap();
    assert_eq!(&buffer, b"Hello, world!");
}

#[test]
#[tracing_test::traced_test]
fn test_decompressing_reader_read_n_bytes_too_few_bytes() {
    let data = b"Hello, world!";
    let mut compressed_data = Vec::new();
    {
        let mut compressor =
            CompressingWriter::builder(&mut compressed_data).build();
        // Only compress some of the message, to simulate a partial
        // write/compress of the data
        compressor.write_all(&data[1..3]).unwrap();
        compressor.flush().unwrap();
    }

    let mut compressed_data_cursor = Cursor::new(&compressed_data);
    let mut decompressor =
        DecompressingReader::builder(&mut compressed_data_cursor).build();
    let mut buffer = vec![0u8; data.len()];
    let result = decompressor.read_exact(&mut buffer);
    assert!(result.is_err());
    let result = result.unwrap_err();
    assert_eq!(result.kind(), std::io::ErrorKind::UnexpectedEof);
}
