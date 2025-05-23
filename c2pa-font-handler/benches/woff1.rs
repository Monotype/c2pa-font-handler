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

#![allow(missing_docs)]
//#![cfg(feature = "woff")]

#[path = "utils/profiler.rs"]
mod profiler_utils;
use c2pa_font_handler::{
    chunks::ChunkReader,
    sfnt::table::TableC2PA,
    woff1::{
        directory::{Woff1Directory, Woff1DirectoryEntry},
        font::Woff1Font,
        header::Woff1Header,
    },
    FontDataExactRead, FontDataRead, FontDataWrite,
};
use criterion::{criterion_group, criterion_main, Criterion};
use profiler_utils::DhatProfiler;

#[path = "utils/c2pa.rs"]
mod c2pa_utils;
use c2pa_utils::*;
#[path = "utils/woff1.rs"]
mod woff1_utils;
use woff1_utils::*;

fn woff1_font_benchmarks(c: &mut Criterion) {
    // Benchmark for reading the font data
    c.bench_function("parse_woff1_from_reader", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(get_woff1_font_data());
            let _ = Woff1Font::from_reader(&mut font_stream)
                .expect("Failed to read font data");
        });
    });

    // Benchmark the process of getting chunk positions
    c.bench_function("get_chunk_positions", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(get_woff1_font_data());
            let _ = Woff1Font::get_chunk_positions(&mut font_stream)
                .expect("Failed to get chunk positions");
        });
    });

    /* // UNCOMMENT THIS BLOCK TO TEST ONCE WOFF1 supports updating C2PA
    // Benchmark the process of mimicking the C2PA read/write process of a
    // typical signing flow.
    c.bench_function("woff1_c2pa_readwrite_mimic", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(get_woff1_font_data());
            let mut dest_stream = std::io::Cursor::new(Vec::new());

            // Simulate the removal of the C2PA record
            parse_font_and_execute_write(&mut font_stream, Some(&mut dest_stream), |font| {
                if font.has_c2pa() {
                   font.remove_c2pa_record()
                       .expect("Failed to remove C2PA record");
                }
                let update_record = c2pa_font_handler::c2pa::UpdateContentCredentialRecord::builder()
                    .with_active_manifest_uri("urn:monotype:example".to_string())
                    .build();
                font.update_c2pa_record(update_record)
                    .expect("Failed to update C2PA record");
            });

            // Get the chunk positions, done by the SDK to determine general box
            // positions
            font_stream.set_position(0);
            let _ = Woff1Font::get_chunk_positions(&mut font_stream)
                .expect("Failed to get chunk positions");

            // Simulate adding new C2PA record
            let mut dest_stream = std::io::Cursor::new(Vec::new());
            font_stream.set_position(0);
            parse_font_and_execute_write(&mut font_stream, Some(&mut dest_stream), |font| {
                let record = c2pa_font_handler::c2pa::UpdateContentCredentialRecord::builder()
                    .with_active_manifest_uri("urn:monotype:example".to_string())
                    .build();
                font.update_c2pa_record(record)
                    .expect("Failed to add C2PA record");
            });

            // Simulate reading C2PA for validation
            let mut font_stream =
                std::io::Cursor::new(dest_stream.get_ref().as_slice());
            parse_font_and_execute(&mut font_stream, |font| {
                font.get_c2pa().expect("Failed to read C2PA record");
            });
        });
    });
    */
}

/// Collection of benchmarks for WOFF1 font header reading and writing.
fn woff1_header_benchmarks(c: &mut Criterion) {
    c.bench_function("woff1_header_read", |b| {
        b.iter(|| {
            let header_data = get_woff1_header_data();
            let mut font_stream = std::io::Cursor::new(header_data);
            let _ = Woff1Header::from_reader_exact(
                &mut font_stream,
                0,
                header_data.len(),
            )
            .expect("Failed to read WOFF1 font data");
        });
    });
    c.bench_function("woff1_header_write", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(Vec::new());
            let header = get_woff1_header();
            header
                .write(&mut font_stream)
                .expect("Failed to write WOFF1 header");
        });
    });
}

/// Collection of benchmarks for WOFF1 font directory data reading and writing.
fn woff1_directory_benchmarks(c: &mut Criterion) {
    c.bench_function("woff1_directory_read", |b| {
        b.iter(|| {
            let data = get_woff1_directory_data();
            let mut font_stream = std::io::Cursor::new(data);
            let _ = Woff1Directory::from_reader_exact(
                &mut font_stream,
                0,
                size_of::<Woff1DirectoryEntry>() * FONT_WOFF1_DIRECTORY_ENTRIES,
            )
            .expect("Failed to read WOFF1 font data");
        });
    });
    c.bench_function("woff1_directory_write", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(Vec::new());
            let directory = get_woff1_directory();
            directory
                .write(&mut font_stream)
                .expect("Failed to write WOFF1 directory");
        });
    });
}

/// Collection of benchmarks for WOFF1 table reading and writing.
fn woff1_table_benchmarks(c: &mut Criterion) {
    c.bench_function("woff1_table_read", |b| {
        b.iter(|| {
            let (data, length) = get_faux_c2pa_table_data();
            let mut font_stream = std::io::Cursor::new(data);
            let _ = TableC2PA::from_reader_exact(&mut font_stream, 0, length)
                .expect("Failed to read WOFF1 font data");
        });
    });
    c.bench_function("woff1_table_write", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(Vec::new());
            let table = get_faux_c2pa_table();
            table
                .write(&mut font_stream)
                .expect("Failed to write WOFF1 table");
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_profiler(DhatProfiler::new());
    targets =  woff1_table_benchmarks, woff1_directory_benchmarks, woff1_header_benchmarks, woff1_font_benchmarks,
);
criterion_main!(benches);
