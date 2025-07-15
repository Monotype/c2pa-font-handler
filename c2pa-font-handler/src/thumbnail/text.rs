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

#![allow(dead_code)]

//! This module provides functionality for using the cosmic-text crate
//! to create a font system for a given font, with no fallback fonts. This is
//! used to generate thumbnails for fonts, which can be used in C2PA
//! operations.

use std::{
    io::{Read, Seek},
    sync::Arc,
};

use cosmic_text::{
    fontdb::{Database, ID},
    ttf_parser::{name_id, PlatformId},
    Attrs, BorrowedWithFontSystem, Buffer, CacheKeyFlags, Fallback, Font,
    FontFeatures, FontSystem, Metrics, SwashCache,
};

use super::{
    error::FontThumbnailError, ReadSeek, Renderer, ThumbnailGenerator,
};
use crate::mime_type;

/// Context for the text font system, which includes the font system, swash
/// cache, text buffer, and the angle of the font if it is italic.
#[derive(Debug)]
pub struct TextFontSystemContext {
    /// The font system to use for rendering text
    pub font_system: FontSystem,
    /// The swash cache to use for rendering text
    pub swash_cache: SwashCache,
    /// The text buffer to use for rendering text
    pub text_buffer: Buffer,
    /// The angle of the font, if it is italic
    pub angle: Option<f32>,
}

impl TextFontSystemContext {
    /// Get the angle of the font if it is italic, otherwise None
    pub fn angle(&self) -> Option<f32> {
        self.angle
    }

    /// Get the cosmic-text parts from the context
    pub fn mut_cosmic_text_parts(
        &mut self,
    ) -> (&mut FontSystem, &mut SwashCache, &mut Buffer) {
        (
            &mut self.font_system,
            &mut self.swash_cache,
            &mut self.text_buffer,
        )
    }
}

/// A thumbnail generator that uses the cosmic-text crate to render text
/// thumbnails. This generator is designed to create thumbnails for fonts
/// without any fallback fonts, which is useful for C2PA operations where
/// fallback fonts are not desired.
pub struct CosmicTextThumbnailGenerator<'a> {
    /// The underlying renderer that will be used to render the thumbnail
    renderer: Box<dyn Renderer>,
    /// The font system configuration to use for the thumbnail generation
    font_system_config: FontSystemConfig<'a>,
}

impl<'a> CosmicTextThumbnailGenerator<'a> {
    /// Create a new SVG thumbnail generator with the given renderer.
    pub fn new(render: Box<dyn Renderer>) -> Self {
        Self {
            renderer: render,
            font_system_config: FontSystemConfig::default(),
        }
    }

    /// Create a new SVG thumbnail generator with the given renderer.
    pub fn new_with_config(
        renderer: Box<dyn Renderer>,
        font_system_config: FontSystemConfig<'a>,
    ) -> Self {
        Self {
            renderer,
            font_system_config,
        }
    }
}

impl<'a> ThumbnailGenerator for CosmicTextThumbnailGenerator<'a> {
    fn create_thumbnail_from_stream(
        &self,
        reader: &mut dyn ReadSeek,
        mime_type: Option<&str>,
    ) -> Result<super::Thumbnail, super::error::FontThumbnailError> {
        // Determine the MIME type, guessing if not provided
        let mime = match mime_type {
            Some(m) => m.to_owned(),
            None => {
                tracing::trace!("Guessing MIME type for font data");
                mime_type::MimeTypeGuesser::guess_mime_type(reader)
                    .map_err(FontThumbnailError::from)?
            }
        };
        tracing::trace!("Attempting to generate thumbnail for source data with MIME type: {mime}");

        match mime.as_str() {
            mime_type::MimeTypes::OTF | mime_type::MimeTypes::TTF => {
                tracing::trace!("Creating font system from SFNT data");
                let mut context =
                    create_font_system(&self.font_system_config, reader)?;
                tracing::trace!("Rendering thumbnail for SFNT font");
                self.renderer.render_thumbnail(&mut context)
            }
            #[cfg(feature = "woff")]
            mime_type::MimeTypes::WOFF | mime_type::MimeTypes::WOFF2 => {
                use std::io::Cursor;

                use crate::{
                    sfnt::font::SfntFont, FontDataRead, MutFontDataWrite,
                };
                tracing::trace!("Converting WOFF/WOFF2 to SFNT");
                // Parse WOFF/WOFF2, convert to SFNT, and render
                let woff_font =
                    crate::woff1::font::Woff1Font::from_reader(reader)?;
                let mut sfnt_font = SfntFont::try_from(woff_font)?;

                // Write SFNT font to an in-memory buffer
                let mut font_buf = Vec::new();
                sfnt_font.write(&mut font_buf)?;

                tracing::trace!("Creating font system from SFNT data created from WOFF/WOFF2");
                let mut cursor = Cursor::new(font_buf);
                let mut context =
                    create_font_system(&self.font_system_config, &mut cursor)?;
                tracing::trace!("Rendering thumbnail for WOFF/WOFF2 font");
                self.renderer.render_thumbnail(&mut context)
            }
            _ => Err(FontThumbnailError::UnsupportedInputMimeType),
        }
    }
}

/// Information about the font
struct FontNameInfo {
    /// Full name of the font
    full_name: Option<String>,
    /// Sample text for the font
    #[allow(unused)]
    sample_text: Option<String>,
}

impl From<Arc<Font>> for FontNameInfo {
    fn from(font: Arc<Font>) -> Self {
        // The US English language ID; currently localization is still a work in
        // progress
        const US_EN_LANGUAGE_ID: u16 = 0x0409;
        // The Unicode BMP encoding
        const UNICODE_BMP_ENCODING: u16 = 3;
        // The Windows Symbol encoding
        const WINDOWS_SYMBOL_ENCODING: u16 = 0;
        // The Windows BMP encoding
        const WINDOWS_BMP_ENCODING: u16 = 1;

        let face = font.rustybuzz();
        // We want to use PlatformID::Unicode/LanguageID::English for the name
        // table when possible, if not available, we will look for
        // Windows, and then finally Macintosh
        let preferred_search_order = [
            (PlatformId::Unicode, US_EN_LANGUAGE_ID, UNICODE_BMP_ENCODING),
            (
                PlatformId::Windows,
                US_EN_LANGUAGE_ID,
                WINDOWS_SYMBOL_ENCODING,
            ),
            (PlatformId::Windows, US_EN_LANGUAGE_ID, WINDOWS_BMP_ENCODING),
        ];

        let find_name = |name_id: u16| {
            preferred_search_order
                .iter()
                .find_map(|&(platform, lang, encoding)| {
                    face.names().into_iter().find(|n| {
                        n.name_id == name_id
                            && n.platform_id == platform
                            && n.language_id == lang
                            && n.encoding_id == encoding
                    })
                })
                .and_then(|name| name.to_string())
        };

        let full_name = find_name(name_id::FULL_NAME);
        let sample_text = find_name(name_id::SAMPLE_TEXT);

        FontNameInfo {
            full_name,
            sample_text,
        }
    }
}

/// Information about a loaded font, including its ID and attributes.
#[derive(Debug)]
struct LoadedFont<'a> {
    /// The ID of the loaded font in the font database
    id: ID,
    /// The attributes of the loaded font
    attrs: Attrs<'a>,
}

/// Load font data into the font database, returning the ID of the loaded font
fn load_font_data<'a>(
    font_db: &mut Database,
    font_data: Vec<u8>,
) -> Result<LoadedFont<'a>, FontThumbnailError> {
    font_db.load_font_data(font_data);
    let face = font_db
        .faces()
        .last()
        .ok_or(FontThumbnailError::NoFontFound)?;
    let weight = face.weight;
    let style = face.style;
    let stretch = face.stretch;
    let attrs: Attrs = Attrs {
        color_opt: None,
        family: cosmic_text::Family::Serif,
        stretch,
        style,
        weight,
        metadata: 0,
        cache_key_flags: CacheKeyFlags::empty(),
        metrics_opt: None,
        letter_spacing_opt: None,
        font_features: FontFeatures::new(),
    };
    Ok(LoadedFont { id: face.id, attrs })
}

/// The cosmic-text crate provides a [`Fallback`] trait that is used to provide
/// fallback fonts In our scenario, we do not want to use any fallback fonts for
/// generating the thumbnail, so we implement a no-op version of the trait
#[derive(Default)]
struct NoFallback {}

impl Fallback for NoFallback {
    fn common_fallback(&self) -> &[&'static str] {
        &[]
    }

    fn forbidden_fallback(&self) -> &[&'static str] {
        &[]
    }

    fn script_fallback(
        &self,
        _script: unicode_script::Script,
        _locale: &str,
    ) -> &[&'static str] {
        &[]
    }
}

/// Configuration for the font system used to generate thumbnails
#[derive(Debug, Clone)]
pub struct FontSystemConfig<'a> {
    /// The default locale to use for the font system
    default_locale: &'a str,
    /// The line height factor for the thumbnail
    line_height_factor: f32,
    /// The maximum width for the thumbnail
    maximum_width: u32,
    /// The minimum point size for the font system
    minimum_point_size: f32,
    /// The step size to reduce the point size when searching for a fitting
    /// width
    point_size_step: f32,
    /// The starting point size for the font system
    starting_point_size: f32,
    /// The total width padding to apply to the thumbnail
    total_width_padding: f32,
}

impl FontSystemConfig<'static> {
    /// Default locale for the font system
    const DEFAULT_LOCALE: &'static str = "en-US";
    /// The line height factor for the thumbnail
    const LINE_HEIGHT_FACTOR: f32 = 1.075;
    /// Maximum width for the thumbnail
    const MAXIMUM_WIDTH: u32 = 400;
    /// Minimum point size for the font system
    const MINIMUM_POINT_SIZE: f32 = 6.0;
    /// Step size to reduce the point size when searching for a fitting width
    const POINT_SIZE_STEP: f32 = 8.0;
    /// Starting point size for the font system
    const STARTING_POINT_SIZE: f32 = 512.0;
    /// Total width padding to apply to the thumbnail
    const TOTAL_WIDTH_PADDING: f32 = 0.1; // 10% padding
}

impl<'a> FontSystemConfig<'a> {
    /// Create a new font system configuration with the given parameters
    pub fn new(
        default_locale: &'a str,
        line_height_factor: f32,
        maximum_width: u32,
        minimum_point_size: f32,
        point_size_step: f32,
        starting_point_size: f32,
        total_width_padding: f32,
    ) -> Self {
        Self {
            default_locale,
            line_height_factor,
            maximum_width,
            minimum_point_size,
            point_size_step,
            starting_point_size,
            total_width_padding,
        }
    }
}

impl Default for FontSystemConfig<'static> {
    fn default() -> Self {
        Self {
            default_locale: Self::DEFAULT_LOCALE,
            line_height_factor: Self::LINE_HEIGHT_FACTOR,
            maximum_width: Self::MAXIMUM_WIDTH,
            minimum_point_size: Self::MINIMUM_POINT_SIZE,
            point_size_step: Self::POINT_SIZE_STEP,
            starting_point_size: Self::STARTING_POINT_SIZE,
            total_width_padding: Self::TOTAL_WIDTH_PADDING,
        }
    }
}

/// For a given stream of font data, create a font system and a buffer that fits
/// the given width and height, ready for rendering.
///
/// # Parameters
/// - `config`: The configuration for the font system.
/// - `stream`: The stream containing the font data.
///
/// # Returns
/// A tuple containing the buffer, font system, swash cache, and angle (if
/// italic).
pub fn create_font_system<R: Read + Seek + ?Sized>(
    config: &FontSystemConfig,
    stream: &mut R,
) -> Result<TextFontSystemContext, FontThumbnailError> {
    let font_data =
        std::io::Read::bytes(stream).collect::<std::io::Result<Vec<u8>>>()?;
    // Create a local font database, which only contains the font we loaded
    let mut font_db = Database::new();
    // Load the given font file into the font database, getting the ID of the
    // font to use with the font system
    let loaded_font: LoadedFont = load_font_data(&mut font_db, font_data)?;

    // And build a font system from this local database
    let mut font_system =
        cosmic_text::FontSystem::new_with_locale_and_db_and_fallback(
            config.default_locale.to_string(),
            font_db.clone(),
            NoFallback::default(),
        );
    // Get reference to the font from the font system
    let f = font_system
        .get_font(loaded_font.id)
        .ok_or(FontThumbnailError::NoFontFound)?;
    // Grab the potential italic angle of the font to calculate the width
    // of the slant later
    let angle = f.rustybuzz().italic_angle();
    let font_info = FontNameInfo::from(f.clone());
    let full_name = font_info
        .full_name
        .ok_or(FontThumbnailError::NoFullNameFound)?;

    // Create a swash cache for the font system, to cache rendering
    let swash_cache = SwashCache::new();

    let ascender = f.rustybuzz().ascender() as i32;
    let descender = f.rustybuzz().descender() as i32;
    let max_height: f32 =
        (ascender - descender) as f32 / f.rustybuzz().units_per_em() as f32;

    // Find a buffer that fits the width
    let buffer = get_buffer_with_pt_size_fits_width(
        &full_name,
        loaded_font.attrs.clone(),
        &mut font_system,
        config,
        |x| (max_height * config.line_height_factor * x).ceil(),
    )?;

    //Ok((buffer, font_system, swash_cache, angle))
    Ok(TextFontSystemContext {
        font_system,
        swash_cache,
        text_buffer: buffer,
        angle,
    })
}

/// Finds the point size that fits the width and creates a buffer with the text
/// and has it ready for rendering.
///
/// # Remarks
/// The `line_height_fn` is a function that takes the font size and should be
/// used to calculate the line height.
fn get_buffer_with_pt_size_fits_width<T: Fn(f32) -> f32>(
    text: &str,
    attrs: Attrs,
    font_system: &mut FontSystem,
    config: &FontSystemConfig,
    line_height_fn: T,
) -> Result<Buffer, FontThumbnailError> {
    // Starting point size
    let mut font_size: f32 = config.starting_point_size;
    // Generate the line height from the font height
    let mut line_height: f32 = line_height_fn(font_size);

    // Make sure there is a enough room for line wrapping to account for the
    // width being too small
    let height = line_height * 2.5;
    let width =
        config.maximum_width as f32 * (1.0 - config.total_width_padding);

    // Create a buffer for measuring the text
    let mut buffer =
        Buffer::new(font_system, Metrics::new(font_size, line_height));
    let mut borrowed_buffer = buffer.borrow_with(font_system);

    // Loop until we find the right size to fit within the maximum width
    while font_size > config.minimum_point_size {
        borrowed_buffer.set_size(Some(width), Some(height));
        borrowed_buffer.set_wrap(cosmic_text::Wrap::Glyph);
        borrowed_buffer.set_text(text, &attrs, cosmic_text::Shaping::Advanced);
        borrowed_buffer.shape_until_scroll(true);
        // Get the number of layout runs, we expect one if it fits on one line
        let count = borrowed_buffer.layout_runs().count();
        // If it is one, we have found the right size
        if count == 1 {
            let size = measure_text(text, &attrs, &mut borrowed_buffer)?;
            // There instances where the measured width was 0, but maybe this is
            // caught now by counting the number of layout runs?
            if size.w > 0.0 && size.w <= width {
                borrowed_buffer.set_size(Some(size.w), Some(size.h));
                return Ok(buffer);
            }
        }
        // Adjust and prepare to try again
        font_size -= config.point_size_step;
        line_height = line_height_fn(font_size);

        // Update the buffer with the new font size
        borrowed_buffer.set_metrics(Metrics::new(font_size, line_height));
    }
    // At this point we have reached our minimum size, so setup to use it
    // which will result in text clipping, but that is fine
    font_size = config.minimum_point_size;
    line_height = line_height_fn(font_size);
    borrowed_buffer.set_size(Some(width), Some(line_height));
    borrowed_buffer.set_metrics(Metrics::new(font_size, line_height));
    borrowed_buffer.shape_until_scroll(true);
    // get the text replacing the last 3 characters with ellipsis
    let text = if text.len() > 3 {
        format!("{}...", text.split_at(text.len() - 3).0)
    } else {
        text.to_string()
    };
    borrowed_buffer.set_text(&text, &attrs, cosmic_text::Shaping::Advanced);
    let size = measure_text(&text, &attrs, &mut borrowed_buffer)?;
    // We still run the chance of an invalid size returned, so take that into
    // account
    if size.w > 0.0 && size.w <= width && size.h <= height {
        borrowed_buffer.set_size(Some(size.w), Some(size.h));
        return Ok(buffer);
    }
    Err(FontThumbnailError::FailedToFindAppropriateSize)
}

/// Size of the bounding box
#[derive(Debug, Default, Clone)]
struct Size {
    /// Width of the bounding box
    w: f32,
    /// Height of the bounding box
    h: f32,
}

/// Measure the text to get the size of the bounding box required
///
/// # Remarks
/// The width may come back as `0.0` if the text is empty or the buffer width is
/// too small.
fn measure_text(
    text: &str,
    attrs: &Attrs,
    buffer: &mut BorrowedWithFontSystem<Buffer>,
) -> Result<Size, FontThumbnailError> {
    buffer.set_text(text, attrs, cosmic_text::Shaping::Advanced);
    buffer.shape_until_scroll(true);
    measure_text_in_buffer(buffer)
}

/// Measure the text in the buffer to get the size of the bounding box required
///
/// # Remarks
/// The width may come back as `0.0` if the text is empty or the buffer width is
/// too small.
fn measure_text_in_buffer(
    buffer: &mut BorrowedWithFontSystem<Buffer>,
) -> Result<Size, FontThumbnailError> {
    let buffer_width = buffer.size().0.unwrap_or(-1.0);
    let buffer_height = buffer.size().1.unwrap_or(-1.0);

    // Error if the buffer is not a valid/sane looking size
    if buffer_width < 0.0 || buffer_height < 0.0 {
        return Err(FontThumbnailError::InvalidBufferSize);
    }
    // Find the maximum width of the layout lines and keep track of the total
    // number of lines
    let (width, total_lines) = buffer
        .layout_runs()
        .fold((0.0, 0usize), |(width, total_lines), run| {
            (run.line_w.max(width), total_lines + 1)
        });
    Ok(Size {
        w: width,
        h: total_lines as f32 * buffer.metrics().line_height,
    })
}

#[cfg(test)]
#[path = "./text_test.rs"]
mod tests;
