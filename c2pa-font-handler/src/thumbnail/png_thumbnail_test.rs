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

//! Tests for PNG thumbnail generation.

use std::io::Cursor;

use super::*;
use crate::thumbnail::text::{create_font_system, FontSystemConfig};

/// Sets up a test context with a dummy font system and swash cache.
fn setup_cosmic_text_for_test() -> TextFontSystemContext {
    let mut font_data =
        Cursor::new(include_bytes!("../../../.devtools/font.otf"));
    create_font_system(&FontSystemConfig::default(), &mut font_data).unwrap()
}

#[test]
fn test_get_skia_paint_for_color() {
    let color = cosmic_text::Color::rgba(255, 0, 0, 255);
    let skia_paint_color = get_skia_paint_for_color(color);
    assert_eq!(skia_paint_color.blend_mode, tiny_skia::BlendMode::Color);
    assert!(skia_paint_color.anti_alias);
    assert!(matches!(
        skia_paint_color.shader,
        tiny_skia::Shader::SolidColor(_)
    ));
}

#[test]
fn test_default_png_thumbnail_renderer_config() {
    let config = PngThumbnailRendererConfig::default();
    assert_eq!(
        config.text_color,
        PngThumbnailRendererConfig::DEFAULT_TEXT_COLOR
    );
    assert_eq!(
        config.background_color,
        PngThumbnailRendererConfig::DEFAULT_BACKGROUND_COLOR
    );
}

#[test]
fn test_new_png_thumbnail_renderer_config() {
    let text_color = cosmic_text::Color::rgba(0, 128, 0, 255);
    let background_color = tiny_skia::Color::from_rgba8(255, 255, 0, 255);
    let config = PngThumbnailRendererConfig::new(text_color, background_color);

    assert_eq!(config.text_color, text_color);
    assert_eq!(config.background_color, background_color);
}

#[test]
fn test_default_png_thumbnail_renderer() {
    let renderer = PngThumbnailRenderer::default();
    assert_eq!(
        renderer.config.text_color,
        PngThumbnailRendererConfig::DEFAULT_TEXT_COLOR
    );
    assert_eq!(
        renderer.config.background_color,
        PngThumbnailRendererConfig::DEFAULT_BACKGROUND_COLOR
    );
    let mut context = setup_cosmic_text_for_test();
    let result = renderer.render_thumbnail(&mut context);
    assert!(result.is_ok());
    let thumbnail = result.unwrap();
    assert_eq!(thumbnail.mime_type(), "image/png");
    assert!(!thumbnail.data().is_empty());
    assert!(thumbnail.data().starts_with(b"\x89PNG\r\n\x1a\n"));
}

// Verify the error path when the buffer size is invalid
#[test]
fn test_png_thumbnail_renderer_invalid_size() {
    let config = PngThumbnailRendererConfig::default();
    let renderer = PngThumbnailRenderer::new(config);
    let mut context = setup_cosmic_text_for_test();

    // Setup test parameters
    {
        let (font_system, _swash_cache, buffer) =
            context.mut_cosmic_text_parts();
        buffer.set_size(font_system, None, Some(18.0));
    }
    // Attempt to render a thumbnail with invalid size
    let result = renderer.render_thumbnail(&mut context);

    // Expect an error due to invalid size
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, FontThumbnailError::InvalidBufferSize));

    // Now set invalid height
    {
        let (font_system, _swash_cache, buffer) =
            context.mut_cosmic_text_parts();
        buffer.set_size(font_system, Some(18.0), None);
    }
    // Attempt to render a thumbnail with invalid height
    let result = renderer.render_thumbnail(&mut context);
    // Expect an error due to invalid height
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, FontThumbnailError::InvalidBufferSize));
}

// Verify rendering a PNG when the angle is None
#[test]
fn test_png_thumbnail_renderer_none_angle() {
    let config = PngThumbnailRendererConfig::default();
    let renderer = PngThumbnailRenderer::new(config);
    let mut context = setup_cosmic_text_for_test();

    context.angle = None; // Set angle to None
                          // Attempt to render a thumbnail with invalid size
    let result = renderer.render_thumbnail(&mut context);

    // Expect an error due to invalid size
    assert!(result.is_ok());
    let error = result.unwrap();
    assert_eq!(error.mime_type(), "image/png");
    assert!(!error.data().is_empty());
}
