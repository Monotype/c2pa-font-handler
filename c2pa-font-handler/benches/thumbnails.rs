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
//! Benchmarks for thumbnail generation and handling in C2PA fonts.

#[path = "utils/profiler.rs"]
mod profiler_utils;
use std::io::Cursor;

use c2pa_font_handler::{
    mime_type::FontMimeTypes,
    thumbnail::{
        CosmicTextThumbnailGenerator, PngThumbnailRenderer,
        PngThumbnailRendererConfig, SvgThumbnailRenderer,
        SvgThumbnailRendererConfig, ThumbnailGenerator,
    },
};
use criterion::{criterion_group, criterion_main, Criterion};
use profiler_utils::DhatProfiler;

/// Collection of benchmarks for SFNT font thumbnail generation.
fn sfnt_thumbnail_benchmarks(c: &mut Criterion) {
    let font_data = include_bytes!("../../.devtools/font.otf");

    // Benchmark for generating an SVG thumbnail
    c.bench_function("sfnt_svg_thumbnail_default", |b| {
        b.iter(|| {
            let mut font_stream = Cursor::new(font_data);
            // SVG renderer
            let svg_renderer = Box::new(SvgThumbnailRenderer::new(
                SvgThumbnailRendererConfig::default(),
            ));
            let generator = CosmicTextThumbnailGenerator::new(svg_renderer);
            let _ = generator
                .create_thumbnail_from_stream(
                    &mut font_stream,
                    Some(&FontMimeTypes::OTF),
                )
                .unwrap();
        });
    });

    // Benchmark for generating a PNG thumbnail
    c.bench_function("sfnt_png_thumbnail_default", |b| {
        b.iter(|| {
            let mut font_stream = Cursor::new(font_data);
            // PNG renderer
            let png_renderer = Box::new(PngThumbnailRenderer::new(
                PngThumbnailRendererConfig::default(),
            ));
            let generator = CosmicTextThumbnailGenerator::new(png_renderer);
            let _ = generator
                .create_thumbnail_from_stream(
                    &mut font_stream,
                    Some(&FontMimeTypes::OTF),
                )
                .unwrap();
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_profiler(DhatProfiler::new());
    targets =  sfnt_thumbnail_benchmarks,
);
criterion_main!(benches);
