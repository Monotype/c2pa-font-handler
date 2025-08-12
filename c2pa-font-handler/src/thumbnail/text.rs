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

#[cfg(feature = "woff")]
use std::io::Cursor;
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
use crate::mime_type::{FontMimeTypeGuesser, FontMimeTypes};
#[cfg(feature = "woff")]
use crate::{sfnt::font::SfntFont, FontDataRead, MutFontDataWrite};

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
        mime_type: Option<&FontMimeTypes>,
    ) -> Result<super::Thumbnail, super::error::FontThumbnailError> {
        // Determine the MIME type, guessing if not provided
        let mime = match mime_type {
            Some(m) => m,
            None => {
                tracing::trace!("Guessing MIME type for font data");
                FontMimeTypeGuesser::guess_mime_type(reader)
                    .map_err(FontThumbnailError::from)?
            }
        };
        tracing::trace!("Attempting to generate thumbnail for source data with MIME type: {mime}");

        match mime {
            FontMimeTypes::OTF | FontMimeTypes::TTF => {
                tracing::trace!("Creating font system from SFNT data");
                let mut context =
                    create_font_system(&self.font_system_config, reader)?;
                tracing::trace!("Rendering thumbnail for SFNT font");
                self.renderer.render_thumbnail(&mut context)
            }
            #[cfg(feature = "woff")]
            FontMimeTypes::WOFF => {
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
            _ => {
                tracing::warn!(
                    "Unsupported MIME type for thumbnail generation: {mime}"
                );
                Err(FontThumbnailError::UnsupportedInputMimeType)
            }
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

/// Context for linear font size search parameters
#[derive(Debug, Clone, Copy)]
pub struct LinearSearchContext {
    /// Starting point size for linear search strategy
    pub starting_point_size: f32,
    /// Point size step to reduce the point size when searching for a fitting
    pub point_size_step: f32,
    /// Minimum point size to stop searching at
    pub minimum_point_size: f32,
}

impl LinearSearchContext {
    /// Default minimum point size for linear search strategy
    const DEFAULT_MINIMUM_POINT_SIZE: f32 = 6.0;
    /// Default point size step for linear search strategy
    const DEFAULT_POINT_SIZE_STEP: f32 = 8.0;
    /// Default starting point size for linear search strategy
    const DEFAULT_STARTING_POINT_SIZE: f32 = 512.0;

    /// Create a new linear context with the given parameters
    pub fn new(
        starting_point_size: f32,
        point_size_step: f32,
        minimum_point_size: f32,
    ) -> Self {
        Self {
            starting_point_size,
            point_size_step,
            minimum_point_size,
        }
    }
}
impl Default for LinearSearchContext {
    fn default() -> Self {
        Self {
            starting_point_size: Self::DEFAULT_STARTING_POINT_SIZE,
            point_size_step: Self::DEFAULT_POINT_SIZE_STEP,
            minimum_point_size: Self::DEFAULT_MINIMUM_POINT_SIZE,
        }
    }
}

/// Context for binary font size search parameters
#[derive(Debug, Clone, Copy)]
pub struct BinarySearchContext {
    /// Starting point size for binary search strategy
    pub starting_point_size: f32,
    /// Minimum point size to stop searching at
    pub minimum_point_size: f32,
    /// Maximum point size to stop searching at
    pub maximum_point_size: f32,
}

impl BinarySearchContext {
    /// Default maximum point size for binary search strategy
    const DEFAULT_MAXIMUM_POINT_SIZE: f32 = 512.0;
    /// Default minimum point size for binary search strategy
    const DEFAULT_MINIMUM_POINT_SIZE: f32 = 6.0;
    /// Default starting point size for binary search strategy
    const DEFAULT_STARTING_POINT_SIZE: f32 = 42.0;

    /// Create a new binary context with the given parameters
    pub fn new(
        starting_point_size: f32,
        minimum_point_size: f32,
        maximum_point_size: f32,
    ) -> Self {
        Self {
            starting_point_size,
            minimum_point_size,
            maximum_point_size,
        }
    }
}

impl Default for BinarySearchContext {
    fn default() -> Self {
        Self::new(
            Self::DEFAULT_STARTING_POINT_SIZE,
            Self::DEFAULT_MINIMUM_POINT_SIZE,
            Self::DEFAULT_MAXIMUM_POINT_SIZE,
        )
    }
}

/// Strategy for searching for the appropriate font size for thumbnail
/// rendering.
#[derive(Debug, Clone)]
pub enum FontSizeSearchStrategy {
    /// Try every step from large to small (current behavior).
    Linear(LinearSearchContext),
    /// Use binary search to find the best fitting font size.
    Binary(BinarySearchContext),
    /// Use a fixed font size (no search).
    Fixed(f32),
}

impl FontSizeSearchStrategy {
    /// Creates a linear search font size strategy
    pub fn linear(
        starting_point_size: f32,
        point_size_step: f32,
        minimum_point_size: f32,
    ) -> Self {
        Self::Linear(LinearSearchContext::new(
            starting_point_size,
            point_size_step,
            minimum_point_size,
        ))
    }

    /// Creates a binary search font size strategy
    pub fn binary(
        starting_point_size: f32,
        minimum_point_size: f32,
        maximum_point_size: f32,
    ) -> Self {
        Self::Binary(BinarySearchContext::new(
            starting_point_size,
            minimum_point_size,
            maximum_point_size,
        ))
    }

    /// Creates a fixed font size strategy
    pub fn fixed(size: f32) -> Self {
        Self::Fixed(size)
    }
}

impl Default for FontSizeSearchStrategy {
    fn default() -> Self {
        Self::Binary(BinarySearchContext::default())
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
    /// The total width padding to apply to the thumbnail
    total_width_padding: f32,
    /// The strategy to use for searching for the appropriate font size
    font_size_search_strategy: FontSizeSearchStrategy,
}

impl FontSystemConfig<'static> {
    /// Default locale for the font system
    const DEFAULT_LOCALE: &'static str = "en-US";
    /// The line height factor for the thumbnail
    const LINE_HEIGHT_FACTOR: f32 = 1.075;
    /// Maximum width for the thumbnail
    const MAXIMUM_WIDTH: u32 = 400;
    /// Total width padding to apply to the thumbnail
    const TOTAL_WIDTH_PADDING: f32 = 0.1; // 10% padding
}

impl<'a> FontSystemConfig<'a> {
    /// Create a new font system configuration with the given parameters
    pub fn new(
        default_locale: &'a str,
        line_height_factor: f32,
        maximum_width: u32,
        total_width_padding: f32,
        font_size_search_strategy: FontSizeSearchStrategy,
    ) -> Self {
        Self {
            default_locale,
            line_height_factor,
            maximum_width,
            total_width_padding,
            font_size_search_strategy,
        }
    }

    /// Create a new font system configuration builder
    pub fn builder() -> FontSystemConfigBuilder<'a> {
        FontSystemConfigBuilder::new()
    }
}

impl Default for FontSystemConfig<'static> {
    fn default() -> Self {
        Self::new(
            Self::DEFAULT_LOCALE,
            Self::LINE_HEIGHT_FACTOR,
            Self::MAXIMUM_WIDTH,
            Self::TOTAL_WIDTH_PADDING,
            FontSizeSearchStrategy::default(),
        )
    }
}

/// Builder for the font system configuration, allowing for more flexible
/// configuration of the font system parameters.
#[derive(Default)]
pub struct FontSystemConfigBuilder<'a> {
    /// The default locale to use for the font system
    default_locale: Option<&'a str>,
    /// The line height factor for the thumbnail
    line_height_factor: Option<f32>,
    /// The maximum width for the thumbnail
    maximum_width: Option<u32>,
    /// The total width padding to apply to the thumbnail
    total_width_padding: Option<f32>,
    /// The strategy to use for searching for the appropriate font size
    font_size_search_strategy: Option<FontSizeSearchStrategy>,
}

impl<'a> FontSystemConfigBuilder<'a> {
    /// Create a new font system configuration builder
    fn new() -> Self {
        Self::default()
    }

    /// Set the default locale for the font system
    pub fn default_locale(mut self, locale: &'a str) -> Self {
        self.default_locale = Some(locale);
        self
    }

    /// Set the line height factor for the thumbnail
    pub fn line_height_factor(mut self, factor: f32) -> Self {
        self.line_height_factor = Some(factor);
        self
    }

    /// Set the maximum width for the thumbnail
    pub fn maximum_width(mut self, width: u32) -> Self {
        self.maximum_width = Some(width);
        self
    }

    /// Set the strategy to use for searching for the appropriate font size
    pub fn search_strategy(mut self, strategy: FontSizeSearchStrategy) -> Self {
        self.font_size_search_strategy = Some(strategy);
        self
    }

    /// Set the total width padding to apply to the thumbnail
    pub fn total_width_padding(mut self, padding: f32) -> Self {
        self.total_width_padding = Some(padding);
        self
    }

    /// Build the font system configuration from the builder parameters
    pub fn build(self) -> FontSystemConfig<'a> {
        let default_config = FontSystemConfig::default();
        FontSystemConfig {
            default_locale: self
                .default_locale
                .unwrap_or(default_config.default_locale),
            line_height_factor: self
                .line_height_factor
                .unwrap_or(default_config.line_height_factor),
            maximum_width: self
                .maximum_width
                .unwrap_or(default_config.maximum_width),
            total_width_padding: self
                .total_width_padding
                .unwrap_or(default_config.total_width_padding),
            font_size_search_strategy: self
                .font_size_search_strategy
                .unwrap_or(default_config.font_size_search_strategy),
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

    Ok(TextFontSystemContext {
        font_system,
        swash_cache,
        text_buffer: buffer,
        angle,
    })
}

/// If the string is longer than 3 characters, it will replace the last 3
/// characters with an ellipsis ("..."). Otherwise, it will return the
/// original string.
fn clip_text_to_ellipsis(text: &str) -> String {
    let char_count = text.chars().count();
    // If the text is longer than 3 characters, replace the last 3 with ellipsis
    if char_count > 3 {
        format!(
            "{}...",
            text.chars().take(char_count - 3).collect::<String>()
        )
    } else {
        text.to_string()
    }
}

/// Finds a buffer that fits the given width, using the configured search
/// strategy to determine the font size.
///
/// # Remarks
/// A linear search strategy will try every step from large to small.
fn get_buffer_with_linear_search<T: Fn(f32) -> f32>(
    text: &str,
    attrs: Attrs,
    font_system: &mut FontSystem,
    config: &FontSystemConfig,
    line_height_fn: T,
    linear_search_context: &LinearSearchContext,
) -> Result<Buffer, FontThumbnailError> {
    // Starting point size
    let mut font_size: f32 = linear_search_context.starting_point_size;
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

    while font_size > linear_search_context.minimum_point_size {
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
                let final_font_size = font_size;
                tracing::debug!(
                    text,
                    final_font_size,
                    "Found appropriate size: {size:?}; font size: {final_font_size}"
                );
                borrowed_buffer.set_size(Some(size.w), Some(size.h));
                return Ok(buffer);
            }
        }
        // Adjust and prepare to try again
        font_size -= linear_search_context.point_size_step;
        tracing::debug!("Adjusting font size to {font_size}");
        line_height = line_height_fn(font_size);
        tracing::debug!("Adjusting line height to {line_height}");

        // Update the buffer with the new font size
        borrowed_buffer.set_metrics(Metrics::new(font_size, line_height));
    }
    // At this point we have reached our minimum size, so setup to use it
    // which will result in text clipping, but that is fine
    font_size = linear_search_context.minimum_point_size;
    line_height = line_height_fn(font_size);
    borrowed_buffer.set_size(Some(width), Some(line_height));
    borrowed_buffer.set_metrics(Metrics::new(font_size, line_height));
    borrowed_buffer.shape_until_scroll(true);
    // get the text replacing the last 3 characters with ellipsis
    let text = clip_text_to_ellipsis(text);
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

/// Finds a buffer that fits the given width, using the configured binary search
/// strategy
fn get_buffer_with_binary_search<T: Fn(f32) -> f32>(
    text: &str,
    attrs: Attrs<'_>,
    font_system: &mut FontSystem,
    config: &FontSystemConfig<'_>,
    line_height_fn: T,
    context: &BinarySearchContext,
) -> Result<Buffer, FontThumbnailError> {
    /*
     * With a stated maximum and minimum point size
     * Take the integer midpoint of the maxima/minima
     * Check if the entire text fits on one line within the width
     *   [Yes] Then take midpoint of current point and the maxima, recheck
     * width,         continuing until it doesn't fit and taking the last
     * midpoint   [No] Then take midpoint of current point and the
     * minima, recheck width,        continuing until it doesn't fit and
     * taking the last midpoint
     */
    // Start by defining our highest size as the maximum point size
    let mut high = context.maximum_point_size;
    // And the lowest size as the minimum point size
    let mut low = context.minimum_point_size;
    // Calculate the width from the config, incorporating the total width
    // padding
    let width =
        config.maximum_width as f32 * (1.0 - config.total_width_padding);
    tracing::debug!(
        "Starting binary search for font size in range [{low}, {high}] with width {width}"
    );

    // Keep up with what was the best size
    let mut best_size: Option<(f32, Buffer)> = None;

    const EPSILON: f32 = 1.0; // A small value to avoid infinite loop

    while high - low > EPSILON {
        // Calculate the midpoint of the current range, rounding to the nearest
        // integer to avoid floating point precision issues
        let mid = ((low + high) / 2.0).round();
        let line_height: f32 = line_height_fn(mid);
        // Make sure we use a height that is large enough to account for
        // line wrapping
        let height = line_height * 2.5;

        let mut buffer =
            Buffer::new(font_system, Metrics::new(mid, line_height));
        let mut borrowed_buffer = buffer.borrow_with(font_system);

        borrowed_buffer.set_size(Some(width), Some(height));
        borrowed_buffer.set_wrap(cosmic_text::Wrap::Glyph);
        borrowed_buffer.set_text(text, &attrs, cosmic_text::Shaping::Advanced);
        borrowed_buffer.shape_until_scroll(true);
        let line_count = borrowed_buffer.layout_runs().count();
        let size = measure_text(text, &attrs, &mut borrowed_buffer)?;

        if line_count == 1
            && size.w > 0.0
            && size.w <= width
            && size.h <= height
        {
            // It fits, so we will continue by searching for a larger size
            best_size = Some((mid, buffer));
            low = mid;
            tracing::debug!("Text fits at size {mid}, adjusting search range to [{high}, {low}]");
        } else {
            // Otherwise, the text does not fit, so we will search for a smaller
            // size
            high = mid;
            tracing::debug!("Text does not fit at size {mid}, adjusting search range to [{high}, {low}]");
        }
    }
    // If we have a best size, we can use it to create the buffer
    if let Some((final_font_size, mut buffer)) = best_size {
        // We found a size that fits, so we can return it
        let line_height: f32 = line_height_fn(final_font_size);
        let height = line_height;
        let mut borrowed_buffer = buffer.borrow_with(font_system);
        borrowed_buffer.set_size(Some(width), Some(height));
        borrowed_buffer.set_metrics(Metrics::new(final_font_size, line_height));
        borrowed_buffer.set_wrap(cosmic_text::Wrap::Glyph);
        borrowed_buffer.set_text(text, &attrs, cosmic_text::Shaping::Advanced);
        borrowed_buffer.shape_until_scroll(true);
        let size = measure_text(text, &attrs, &mut borrowed_buffer)?;
        tracing::debug!(
            text,
            final_font_size,
            "Found appropriate size: {size:?}; font size: {final_font_size}"
        );
        borrowed_buffer.set_size(Some(size.w), Some(size.h));
        Ok(buffer)
    } else {
        // Otherwise, we did not find a size that fits. So we will use the
        // minimum font size and use the text with ellipsis
        let final_font_size = context.minimum_point_size;
        let line_height: f32 = line_height_fn(final_font_size);
        let height = line_height;
        let mut buffer = Buffer::new(
            font_system,
            Metrics::new(final_font_size, line_height),
        );
        let mut borrowed_buffer = buffer.borrow_with(font_system);
        borrowed_buffer.set_size(Some(width), Some(height));
        borrowed_buffer.set_metrics(Metrics::new(final_font_size, line_height));
        borrowed_buffer.set_wrap(cosmic_text::Wrap::Glyph);
        // get the text replacing the last 3 characters with ellipsis
        let text = clip_text_to_ellipsis(text);
        borrowed_buffer.set_text(&text, &attrs, cosmic_text::Shaping::Advanced);
        borrowed_buffer.shape_until_scroll(true);
        let size = measure_text(&text, &attrs, &mut borrowed_buffer)?;
        // We still run the chance of an invalid size returned, so take that
        // into account
        if size.w > 0.0 && size.w <= width && size.h <= height {
            borrowed_buffer.set_size(Some(size.w), Some(size.h));
            Ok(buffer)
        } else {
            Err(FontThumbnailError::FailedToFindAppropriateSize)
        }
    }
}

/// Creates a buffer with the given fixed size, returning an error if the text
fn get_buffer_with_fixed_size<T: Fn(f32) -> f32>(
    text: &str,
    attrs: Attrs<'_>,
    font_system: &mut FontSystem,
    config: &FontSystemConfig<'_>,
    line_height_fn: T,
    size: &f32,
) -> Result<Buffer, FontThumbnailError> {
    // Generate the line height from the font height
    let line_height: f32 = line_height_fn(*size);

    // Make sure there is a enough room for line wrapping to account for the
    // width being too small
    let height = line_height * 2.5;
    let width =
        config.maximum_width as f32 * (1.0 - config.total_width_padding);

    // Create a buffer for measuring the text
    let mut buffer = Buffer::new(font_system, Metrics::new(*size, line_height));
    let mut borrowed_buffer = buffer.borrow_with(font_system);

    borrowed_buffer.set_size(Some(width), Some(height));
    borrowed_buffer.set_wrap(cosmic_text::Wrap::Glyph);
    borrowed_buffer.set_text(text, &attrs, cosmic_text::Shaping::Advanced);
    borrowed_buffer.shape_until_scroll(true);
    let size = measure_text(text, &attrs, &mut borrowed_buffer)?;
    if size.w > 0.0 && size.w <= width && size.h <= height {
        borrowed_buffer.set_size(Some(size.w), Some(size.h));
        return Ok(buffer);
    }
    Err(FontThumbnailError::FailedToFindAppropriateSize)
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
    match &config.font_size_search_strategy {
        FontSizeSearchStrategy::Linear(ctx) => get_buffer_with_linear_search(
            text,
            attrs,
            font_system,
            config,
            line_height_fn,
            ctx,
        ),
        FontSizeSearchStrategy::Binary(ctx) => get_buffer_with_binary_search(
            text,
            attrs,
            font_system,
            config,
            line_height_fn,
            ctx,
        ),
        FontSizeSearchStrategy::Fixed(size) => get_buffer_with_fixed_size(
            text,
            attrs,
            font_system,
            config,
            line_height_fn,
            size,
        ),
    }
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
