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

#[path = "utils/profiler.rs"]
mod profiler_utils;
use profiler_utils::DhatProfiler;
#[path = "utils/c2pa.rs"]
mod c2pa_utils;
use c2pa_utils::*;

#[path = "utils/sfnt.rs"]
mod sfnt_utils;
use c2pa_font_handler::{
    c2pa::{C2PASupport, UpdatableC2PA},
    chunks::ChunkReader,
    sfnt::{
        directory::{SfntDirectory, SfntDirectoryEntry},
        font::{stub_dsig_stream, SfntFont},
        header::SfntHeader,
        table::TableC2PA,
    },
    FontDSIGStubber, FontDataExactRead, FontDataRead, FontDataWrite,
};
use criterion::{criterion_group, criterion_main, Criterion};
use sfnt_utils::*;

/// Collection of benchmarks for SFNT font data reading and writing.
fn sfnt_font_benchmarks(c: &mut Criterion) {
    // Benchmark for reading the font data
    c.bench_function("parse_sfnt_from_reader", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(get_sfnt_font_data());
            let _ = SfntFont::from_reader(&mut font_stream)
                .expect("Failed to read font data");
        });
    });

    // Benchmark the process of getting chunk positions
    c.bench_function("get_chunk_positions", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(get_sfnt_font_data());
            let _ = SfntFont::get_chunk_positions(&mut font_stream)
                .expect("Failed to get chunk positions");
        });
    });

    // Benchmark the process of mimicking the C2PA read/write process of a
    // typical signing flow.
    c.bench_function("sfnt_c2pa_readwrite_mimic", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(get_sfnt_font_data());
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

            // Get the chunk positions, done by the SDK to determine general box positions
            font_stream.set_position(0);
            let _ = SfntFont::get_chunk_positions(&mut font_stream)
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
            let mut font_stream = std::io::Cursor::new(dest_stream.get_ref().as_slice());
            parse_font_and_execute(&mut font_stream, |font| {
                font.get_c2pa()
                    .expect("Failed to read C2PA record");
            });
        });
    });
}

/// Collection of benchmarks for SFNT header reading and writing.
fn sfnt_header_benchmarks(c: &mut Criterion) {
    let sfnt_data = get_sfnt_header_data();

    c.bench_function("sfnt_header_from_reader", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(sfnt_data);
            let _ = SfntHeader::from_reader(&mut font_stream)
                .expect("Failed to read SFNT header");
        });
    });

    c.bench_function("sfnt_header_from_reader_exact", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(sfnt_data);
            let _header =
                SfntHeader::from_reader_exact(&mut font_stream, 0, 12)
                    .expect("Failed to read SFNT header");
        });
    });

    c.bench_function("sfnt_header_write", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(Vec::new());
            let header = get_sfnt_header();
            header
                .write(&mut font_stream)
                .expect("Failed to write SFNT header");
        });
    });
}

/// Collection of benchmarks for SFNT directory reading and writing.
fn sfnt_directory_benchmarks(c: &mut Criterion) {
    c.bench_function("sfnt_directory_from_reader", |b| {
        b.iter(|| {
            let mut font_stream =
                std::io::Cursor::new(get_sfnt_directory_data());
            let _ = SfntDirectory::from_reader_exact(
                &mut font_stream,
                0,
                size_of::<SfntDirectoryEntry>() * FONT_OTF_DIRECTORY_ENTRIES,
            )
            .expect("Failed to read SFNT font data");
        });
    });
    c.bench_function("sfnt_directory_write", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(Vec::new());
            let directory = get_sfnt_directory();
            directory
                .write(&mut font_stream)
                .expect("Failed to write SFNT directory");
        });
    });
}

/// Collection of benchmarks for SFNT table reading and writing.
fn sfnt_table_benchmarks(c: &mut Criterion) {
    c.bench_function("sfnt_table_read", |b| {
        b.iter(|| {
            let (data, length) = get_faux_c2pa_table_data();
            let mut font_stream = std::io::Cursor::new(data);
            let _ = TableC2PA::from_reader_exact(&mut font_stream, 0, length)
                .expect("Failed to read SFNT font data");
        });
    });
    c.bench_function("sfnt_table_write", |b| {
        b.iter(|| {
            let mut font_stream = std::io::Cursor::new(Vec::new());
            let table = get_faux_c2pa_table();
            table
                .write(&mut font_stream)
                .expect("Failed to write SFNT table");
        });
    });
}

fn sfnt_stub_dsig(c: &mut Criterion) {
    c.bench_function("sfnt_stub_dsig", |b| {
        let mut font_stream = std::io::Cursor::new(get_sfnt_font_data());
        b.iter(|| {
            font_stream.set_position(0);
            let mut dest_stream = std::io::Cursor::new(Vec::new());

            // Simulate the removal of the DSIG record
            parse_font_and_execute_write(
                &mut font_stream,
                Some(&mut dest_stream),
                |font| {
                    font.stub_dsig().expect("Failed to stub DSIG record");
                },
            );
        });
    });
    c.bench_function("sfnt_stub_dsig_stream", |b| {
        let mut font_stream = std::io::Cursor::new(get_sfnt_font_data());
        b.iter(|| {
            font_stream.set_position(0);
            let mut dest_stream = std::io::Cursor::new(Vec::new());
            // Simulate the removal of the DSIG record
            stub_dsig_stream(&mut font_stream, &mut dest_stream)
                .expect("Failed to stub DSIG stream");
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_profiler(DhatProfiler::new());
    targets =  sfnt_font_benchmarks, sfnt_header_benchmarks, sfnt_directory_benchmarks, sfnt_table_benchmarks, sfnt_stub_dsig,
);
criterion_main!(benches);
