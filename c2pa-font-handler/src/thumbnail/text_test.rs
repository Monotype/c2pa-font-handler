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

//! Tests for the text portion of font thumbnails.

use std::io::Cursor;

use cosmic_text::{fontdb::Database, Buffer, Fallback, FontSystem, Metrics};

use super::{
    create_font_system, measure_text, measure_text_in_buffer, NoFallback,
};
use crate::thumbnail::{
    error::FontThumbnailError,
    text::{load_font_data, FontNameInfo, FontSystemConfig, LoadedFont},
    CosmicTextThumbnailGenerator, ThumbnailGenerator,
};

// Test converting a Arc<Font> to a FontNameInfo
#[test]
fn test_font_name_info_conversion() {
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut font_database = Database::new();
    let LoadedFont { id: font_id, .. } =
        load_font_data(&mut font_database, font_data.to_vec()).unwrap();
    let mut font_system = FontSystem::new_with_locale_and_db(
        "en-US".to_string(),
        font_database.clone(),
    );
    let font = font_system.get_font(font_id).unwrap();

    let font_name_info: FontNameInfo = FontNameInfo::from(font);
    assert_eq!(
        font_name_info.full_name,
        Some("AnEmptyFont Regular".to_string())
    );
    assert_eq!(font_name_info.sample_text, None);
}

// Verify the NoFallback implementation does not provide any fallback scripts.
#[test]
fn test_no_fallback_callbacks() {
    let no_fallback = NoFallback {};
    let result =
        no_fallback.script_fallback(unicode_script::Script::Common, "en-US");
    assert_eq!(result.len(), 0, "Expected no fallback scripts");
    let result = no_fallback.common_fallback();
    assert_eq!(result.len(), 0, "Expected no common fallback scripts");
    let result = no_fallback.forbidden_fallback();
    assert_eq!(result.len(), 0, "Expected no fallback for script");
}

// Check the construction of the FontSystemConfig struct.
#[test]
fn test_new_font_system_config() {
    let config =
        FontSystemConfig::new("en-US", 4.20, 1024, 8.0, 2.3, 7.7, 100.0);
    assert_eq!(
        config.default_locale, "en-US",
        "Expected default locale to be 'en-US'"
    );
    assert_eq!(
        config.line_height_factor, 4.20,
        "Expected line height factor to be 4.20"
    );
    assert_eq!(
        config.maximum_width, 1024,
        "Expected max text width to be 1024"
    );
    assert_eq!(
        config.minimum_point_size, 8.0,
        "Expected minimum point size to be 8.0"
    );
    assert_eq!(
        config.point_size_step, 2.3,
        "Expected point size step to be 2.3"
    );
    assert_eq!(
        config.starting_point_size, 7.7,
        "Expected starting point size to be 7.7"
    );
    assert_eq!(
        config.total_width_padding, 100.0,
        "Expected total width padding to be 100.0"
    );
}

#[test]
fn test_create_font_system() {
    let config = FontSystemConfig::default();
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut stream = Cursor::new(font_data);
    let result = create_font_system(&config, &mut stream);
    assert!(result.is_ok(), "Expected successful font system creation");
}

// We should be able to measure text with a valid text buffer and attributes.
#[test]
fn test_measure_text() {
    let text_str = "Hello, World!";
    let metrics = Metrics::new(10.0, 18.0);
    let mut font_system = FontSystem::new();
    let mut text_buffer = Buffer::new(&mut font_system, metrics);
    let mut buffer = text_buffer.borrow_with(&mut font_system);
    buffer.set_size(Some(30.0), Some(30.0));
    let attrs = cosmic_text::Attrs::new();
    let result = measure_text(text_str, &attrs, &mut buffer);
    assert!(result.is_ok(), "Expected successful measurement");
    let size = result.unwrap();
    let (width, height) = (size.w, size.h);
    assert!(width > 0.0, "Expected width to be greater than 0.0");
    assert!(height > 0.0, "Expected height to be greater than 0.0");
}

#[test]
fn test_create_font_system_with_clipping() {
    let config = FontSystemConfig {
        default_locale: "en-US",
        maximum_width: 100,
        minimum_point_size: 80.0, /* Point size to make sure we do not have
                                   * enough space to fit the text, but can
                                   * clip it. */
        ..Default::default()
    };
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut stream = Cursor::new(font_data);
    let result = create_font_system(&config, &mut stream);
    assert!(result.is_ok(), "Expected successful font system creation with clipping; got error: {result:?}");
    let mut context = result.unwrap();
    assert_eq!(Some(0.0), context.angle());
    let (_font_system, _swash_cache, text_buffer) =
        context.mut_cosmic_text_parts();
    assert!(
        matches!(text_buffer.size(), (Some(_), Some(_))),
        "Expected buffer size to be set, got: {:?}",
        text_buffer.size()
    );
}

#[test]
fn test_create_font_system_failing_to_find_appropriate_size() {
    let config = FontSystemConfig {
        minimum_point_size: 8.0,
        total_width_padding: 100.0,
        ..Default::default()
    };
    let font_data = include_bytes!("../../../.devtools/font.otf");
    let mut stream = Cursor::new(font_data);
    let result = create_font_system(&config, &mut stream);
    assert!(
        result.is_err(),
        "Expected to fail to create font system creation"
    );
    let error = result.unwrap_err();
    assert!(
        matches!(error, FontThumbnailError::FailedToFindAppropriateSize),
        "Expected error to be FailedToFindAppropriateSize; found: {error:?}"
    );
    // Check that the text buffer is created with the correct size
}

// Test the method correctly catches when the text buffer is an incorrect size.
#[test]
fn test_measure_text_in_buffer_invalid_buffer_size() {
    let metrics = Metrics::new(10.0, 18.0);
    let mut font_system = FontSystem::new();
    let mut text_buffer = Buffer::new(&mut font_system, metrics);

    text_buffer.set_size(&mut font_system, None, Some(18.0));
    let mut borrowed_buffer = text_buffer.borrow_with(&mut font_system);

    let result = measure_text_in_buffer(&mut borrowed_buffer);
    assert!(result.is_err(), "Expected an error for invalid buffer size");
    let error = result.unwrap_err();
    assert!(matches!(error, FontThumbnailError::InvalidBufferSize));

    text_buffer.set_size(&mut font_system, Some(18.0), None);
    let mut borrowed_buffer = text_buffer.borrow_with(&mut font_system);

    let result = measure_text_in_buffer(&mut borrowed_buffer);
    assert!(result.is_err(), "Expected an error for invalid buffer size");
    let error = result.unwrap_err();
    assert!(matches!(error, FontThumbnailError::InvalidBufferSize));
}

// Test the method correctly measures text in a valid buffer.
#[test]
fn test_measure_text_in_buffer() {
    let metrics = Metrics::new(10.0, 18.0);
    let mut font_system = FontSystem::new();
    let mut text_buffer = Buffer::new(&mut font_system, metrics);

    // Set the size correctly
    text_buffer.set_size(&mut font_system, Some(18.0), Some(18.0));
    let mut borrowed_buffer = text_buffer.borrow_with(&mut font_system);

    // Measure the text in the buffer
    let result = measure_text_in_buffer(&mut borrowed_buffer);
    assert!(result.is_ok(), "Expected successful measurement");

    // Check that the result is as expected
    let size = result.unwrap();
    let (width, height) = (size.w, size.h);
    // Since we haven't actually used any text, the width should be 0.0
    assert_eq!(width, 0.0, "Expected width to be 0.0");
    assert_eq!(height, 18.0, "Expected height to be 18.0");
}

#[test]
#[tracing_test::traced_test]
fn test_new_cosmic_text_thumbnail_generator() {
    let mut renderer = crate::thumbnail::MockRenderer::new();
    renderer.expect_render_thumbnail().returning(|_| {
        Ok(crate::thumbnail::Thumbnail::new(
            b"<svg></svg>".to_vec(),
            "image/svg+xml".to_string(),
        ))
    });
    let renderer = Box::new(renderer);
    let generator = CosmicTextThumbnailGenerator::new(renderer);
    let mut font_data =
        Cursor::new(include_bytes!("../../../.devtools/font.otf"));
    let result = generator.create_thumbnail_from_stream(&mut font_data, None);
    assert!(result.is_ok(), "Expected successful thumbnail creation");
    let thumbnail = result.unwrap();
    assert_eq!(
        "image/svg+xml",
        thumbnail.mime_type(),
        "Expected mime type to be 'image/svg+xml'"
    );
    assert!(
        !thumbnail.data().is_empty(),
        "Expected thumbnail data to not be empty"
    );
    assert!(
        thumbnail.data().starts_with(b"<svg"),
        "Expected thumbnail data to start with '<svg'"
    );
    assert!(
        logs_contain("Guessing MIME type for font data",),
        "Expected log message about guessing MIME type"
    );
    assert!(logs_contain(
        "Attempting to generate thumbnail for source data with MIME type: font/otf"
    ), "Expected log message about generating thumbnail for font/otf");
    assert!(
        logs_contain("Creating font system from SFNT data"),
        "Expected log message about creating font system from SFNT data"
    );
    assert!(
        logs_contain("Rendering thumbnail for SFNT font"),
        "Expected log message about rendering thumbnail for SFNT font"
    );
}

#[test]
#[tracing_test::traced_test]
fn test_new_cosmic_text_thumbnail_generator_with_unsupported_mime_type() {
    let mut renderer = crate::thumbnail::MockRenderer::new();
    renderer.expect_render_thumbnail().returning(|_| {
        Ok(crate::thumbnail::Thumbnail::new(
            b"<svg></svg>".to_vec(),
            "image/svg+xml".to_string(),
        ))
    });
    let renderer = Box::new(renderer);
    let generator = CosmicTextThumbnailGenerator::new(renderer);
    let mut font_data =
        Cursor::new(include_bytes!("../../../.devtools/font.otf"));
    let result = generator.create_thumbnail_from_stream(
        &mut font_data,
        Some("unsupported/mime-type"),
    );
    assert!(result.is_err(), "Expected error for unsupported mime type");
    let error = result.unwrap_err();
    assert!(
        matches!(error, FontThumbnailError::UnsupportedInputMimeType),
        "Expected error to be UnsupportedInputMimeType; found: {error:?}"
    );
    assert!(logs_contain(
        "Attempting to generate thumbnail for source data with MIME type: unsupported/mime-type"
    ), "Expected log message about unsupported MIME type");
}

#[cfg(feature = "woff")]
#[test]
#[tracing_test::traced_test]
fn test_new_cosmic_text_thumbnail_generator_for_woff() {
    let mut renderer = crate::thumbnail::MockRenderer::new();
    renderer.expect_render_thumbnail().returning(|_| {
        Ok(crate::thumbnail::Thumbnail::new(
            b"<svg></svg>".to_vec(),
            "image/svg+xml".to_string(),
        ))
    });
    let renderer = Box::new(renderer);
    let generator = CosmicTextThumbnailGenerator::new(renderer);
    let mut font_data =
        Cursor::new(include_bytes!("../../../.devtools/font.woff"));
    let result = generator.create_thumbnail_from_stream(&mut font_data, None);
    assert!(result.is_ok(), "Expected successful thumbnail creation");
    let thumbnail = result.unwrap();
    assert_eq!(
        "image/svg+xml",
        thumbnail.mime_type(),
        "Expected mime type to be 'image/svg+xml'"
    );
    assert!(
        !thumbnail.data().is_empty(),
        "Expected thumbnail data to not be empty"
    );
    assert!(
        thumbnail.data().starts_with(b"<svg"),
        "Expected thumbnail data to start with '<svg'"
    );
    assert!(
        logs_contain("Guessing MIME type for font data",),
        "Expected log message about guessing MIME type"
    );
    assert!(logs_contain(
        "Attempting to generate thumbnail for source data with MIME type: font/woff"
    ), "Expected log message about generating thumbnail for font/woff");
    assert!(
        logs_contain("Converting WOFF/WOFF2 to SFNT"),
        "Expected log message about converting WOFF/WOFF2 to SFNT"
    );
    assert!(logs_contain(
        "Creating font system from SFNT data created from WOFF/WOFF2"
    ), "Expected log message about creating font system from SFNT data created from WOFF/WOFF2");
    assert!(
        logs_contain("Rendering thumbnail for WOFF/WOFF2 font"),
        "Expected log message about rendering thumbnail for WOFF/WOFF2 font"
    );
}

#[test]
fn test_cosmic_text_thumbnail_generator_with_font_system_config() {
    let font_system_config = FontSystemConfig::default();
    let mut renderer = crate::thumbnail::MockRenderer::new();
    renderer.expect_render_thumbnail().returning(|_| {
        Ok(crate::thumbnail::Thumbnail::new(
            b"<svg></svg>".to_vec(),
            "image/svg+xml".to_string(),
        ))
    });
    let renderer = Box::new(renderer);
    let generator = CosmicTextThumbnailGenerator::new_with_config(
        renderer,
        font_system_config,
    );
    let mut font_data =
        Cursor::new(include_bytes!("../../../.devtools/font.otf"));
    let result = generator
        .create_thumbnail_from_stream(&mut font_data, Some("font/otf"));
    // Check if the result is Ok
    assert!(result.is_ok());
    let thumbnail = result.unwrap();
    assert_eq!("image/svg+xml", thumbnail.mime_type());
    assert!(!thumbnail.data().is_empty());
    assert!(thumbnail.data().starts_with(b"<svg"));
}

#[test]
fn test_new_cosmic_text_thumbnail_generator_from_path() {
    let mut renderer = crate::thumbnail::MockRenderer::new();
    renderer.expect_render_thumbnail().returning(|_| {
        Ok(crate::thumbnail::Thumbnail::new(
            b"<svg></svg>".to_vec(),
            "image/svg+xml".to_string(),
        ))
    });
    let renderer = Box::new(renderer);
    let generator = CosmicTextThumbnailGenerator::new(renderer);
    // Build up the font path using the Cargo workspace root
    let font_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../.devtools/font.otf");

    let result = generator.create_thumbnail(&font_path);
    assert!(result.is_ok(), "Expected successful thumbnail creation");
    let thumbnail = result.unwrap();
    assert_eq!(
        "image/svg+xml",
        thumbnail.mime_type(),
        "Expected mime type to be 'image/svg+xml'"
    );
    assert!(
        !thumbnail.data().is_empty(),
        "Expected thumbnail data to not be empty"
    );
    assert!(
        thumbnail.data().starts_with(b"<svg"),
        "Expected thumbnail data to start with '<svg'"
    );
}
