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

//! Tests for SVG thumbnail generation.

use std::io::Cursor;

use cosmic_text::{Buffer, FontSystem, SwashCache};

use super::*;
use crate::thumbnail::text::{create_font_system, FontSystemConfig};

struct TestContext {
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_buffer: Buffer,
    angle: Option<f32>,
}
impl TestContext {
    fn into_parts(self) -> (FontSystem, SwashCache, Buffer, Option<f32>) {
        (
            self.font_system,
            self.swash_cache,
            self.text_buffer,
            self.angle,
        )
    }
}

/// Sets up a test context with a dummy font system and swash cache.
fn setup_cosmic_text_for_test() -> TestContext {
    let mut font_data =
        Cursor::new(include_bytes!("../../../.devtools/font.otf"));
    let (text_buffer, font_system, swash_cache, angle) =
        create_font_system(&FontSystemConfig::default(), &mut font_data)
            .unwrap();
    TestContext {
        font_system,
        swash_cache,
        text_buffer,
        angle,
    }
}

#[test]
#[tracing_test::traced_test]
fn test_svg_renderer() {
    // Create a dummy font system and swash cache
    let (mut font_system, mut swash_cache, mut text_buffer, _angle) =
        setup_cosmic_text_for_test().into_parts();

    // Create the SVG thumbnail renderer
    let renderer =
        SvgThumbnailRenderer::new(SvgThumbnailRendererConfig::default());

    // Render a thumbnail using the renderer
    let result = renderer.render_thumbnail(
        &mut text_buffer,
        &mut font_system,
        &mut swash_cache,
    );
    // Check if the result is Ok
    assert!(result.is_ok());

    let thumbnail = result.unwrap();
    assert_eq!("image/svg+xml", thumbnail.mime_type());
    assert!(!thumbnail.data().is_empty());
    assert!(thumbnail.data().starts_with(b"<svg"));
}

#[test]
fn test_svg_renderer_default() {
    // Create a default SVG thumbnail renderer
    let renderer = SvgThumbnailRenderer::default();

    // Check if the default precision is set correctly
    assert_eq!(
        renderer.config.default_precision,
        SvgThumbnailRendererConfig::DEFAULT_SVG_PRECISION
    );
    // Check if the default fill color is set correctly
    assert_eq!(
        renderer.config.glyph_fill_color,
        SvgThumbnailRendererConfig::SVG_GLYPH_FILL_COLOR
    );
}

#[test]
fn test_precision_rounding() {
    // Test rounding for f32
    let value: f32 = 1.234_567_9;
    let rounded_value = value.round_to(3);
    assert_eq!(rounded_value, 1.235);

    // Test rounding for (f32, f32)
    let point: (f32, f32) = (1.234_567_9, 2.345_678_9);
    let rounded_point = point.round_to(3);
    assert_eq!(rounded_point, (1.235, 2.346));
}

#[test]
fn test_default_svg_thumbnail_renderer_config() {
    let config = SvgThumbnailRendererConfig::default();
    assert_eq!(
        config.default_precision,
        SvgThumbnailRendererConfig::DEFAULT_SVG_PRECISION
    );
    assert_eq!(
        config.glyph_fill_color,
        SvgThumbnailRendererConfig::SVG_GLYPH_FILL_COLOR
    );
}
