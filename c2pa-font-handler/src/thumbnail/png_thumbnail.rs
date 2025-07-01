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

//! Thumbnail handling for C2PA fonts using PNG format.

use tiny_skia::Pixmap;

use super::{text::TextFontSystemContext, Renderer};
use crate::thumbnail::error::FontThumbnailError;

/// Get a Skia paint object for the given color.
///
/// This function converts a `cosmic_text::Color` into a `tiny_skia::Paint`
/// object, which can be used for drawing operations in the thumbnail renderer.
///
/// # Remarks
/// The `tiny_skia::Paint` object is configured with a solid color shader,
/// anti-aliasing enabled, and a blend mode set to `Color`. The color is
/// converted from the `cosmic_text::Color` to a `tiny_skia::Color` using
/// the `as_rgba_tuple` method, which provides the RGBA components of the color.
fn get_skia_paint_for_color<'a>(
    color: cosmic_text::Color,
) -> tiny_skia::Paint<'a> {
    tiny_skia::Paint {
        shader: tiny_skia::Shader::SolidColor({
            let (r, g, b, a) = color.as_rgba_tuple();
            tiny_skia::Color::from_rgba8(r, g, b, a)
        }),
        blend_mode: tiny_skia::BlendMode::Color,
        anti_alias: true,
        ..Default::default()
    }
}

/// Configuration for the PNG thumbnail renderer.
#[derive(Debug, Clone)]
pub struct PngThumbnailRendererConfig {
    /// The color to use for the text in the thumbnail.
    text_color: cosmic_text::Color,
    /// The background color for the thumbnail
    background_color: tiny_skia::Color,
}

impl PngThumbnailRendererConfig {
    /// The default background color for the thumbnail
    pub const DEFAULT_BACKGROUND_COLOR: tiny_skia::Color =
        tiny_skia::Color::WHITE;
    /// The default text color for the thumbnail
    pub const DEFAULT_TEXT_COLOR: cosmic_text::Color =
        cosmic_text::Color::rgba(0, 0, 0, 0xff);

    /// Create a new PNG thumbnail renderer configuration
    pub fn new<CT: Into<cosmic_text::Color>, BC: Into<tiny_skia::Color>>(
        text_color: CT,
        background_color: BC,
    ) -> Self {
        Self {
            text_color: text_color.into(),
            background_color: background_color.into(),
        }
    }
}

impl Default for PngThumbnailRendererConfig {
    fn default() -> Self {
        Self::new(
            PngThumbnailRendererConfig::DEFAULT_TEXT_COLOR,
            PngThumbnailRendererConfig::DEFAULT_BACKGROUND_COLOR,
        )
    }
}

/// Renderer for PNG thumbnails from font data.
pub struct PngThumbnailRenderer {
    /// Configuration for the PNG thumbnail renderer.
    config: PngThumbnailRendererConfig,
}

impl PngThumbnailRenderer {
    /// The MIME type for PNG images
    const MIME_TYPE: &'static str = "image/png";

    /// Create a new PNG thumbnail renderer with the given configuration.
    pub fn new(config: PngThumbnailRendererConfig) -> Self {
        Self { config }
    }
}

impl Default for PngThumbnailRenderer {
    fn default() -> Self {
        Self::new(PngThumbnailRendererConfig::default())
    }
}

impl Renderer for PngThumbnailRenderer {
    fn render_thumbnail(
        &self,
        text_system_context: &mut TextFontSystemContext,
    ) -> Result<super::Thumbnail, super::error::FontThumbnailError> {
        let angle = text_system_context.angle;
        let (font_system, swash_cache, text_buffer) =
            text_system_context.mut_cosmic_text_parts();
        // Got some reason, the `swash` library used by `cosmic-text` puts
        // pixels at negative x values, so we need to find the amount we
        // need to offset the image by to include all pixels.
        let layout_run = text_buffer
            .layout_runs()
            .next()
            .ok_or(FontThumbnailError::InvalidBufferSize)?;
        // Grab the cache key for the first glyph, so we can get the image
        // information from the swash cache
        let x = layout_run
            .glyphs
            .first()
            .ok_or(FontThumbnailError::InvalidBufferSize)?
            .physical((0., 0.), 1.0)
            .cache_key;
        let image = swash_cache
            .get_image(font_system, x)
            .as_ref()
            .ok_or(FontThumbnailError::FailedToCreatePixmap)?;
        // Only offset if the left is negative
        let offset = if image.placement.left < 0 {
            image.placement.left.abs()
        } else {
            0
        };

        // Borrow the buffer with the font system, to make things easier to make
        // calls
        let mut buffer = text_buffer.borrow_with(font_system);

        // Grab the actual width and height of the buffer for the image,
        // verifying that the sizes are sane.
        let width = buffer.size().0.unwrap_or(-1.0);
        let height = buffer.size().1.unwrap_or(-1.0);
        if width < 0.0 || height < 0.0 {
            return Err(FontThumbnailError::InvalidBufferSize);
        }

        // Calculate the width taken up by the italic angle
        let width_italic_buffer = match angle {
            // If we have an angle get the tangent of the angle
            Some(angle) => {
                height * ((std::f32::consts::PI / 180.0) * angle).tan()
            }
            // Otherwise, use 0
            _ => 0.0,
        }
        .abs();

        // The total width will be our specified width + the width from the
        // italic angle. QUESTION: Should not the `cosmic-text` library
        // really already include this in the width calculation? Are we doing
        // something wrong?
        // TODO: Investigate whether `cosmic-text` library already includes
        // the italic angle in the width calculation. Verify if this
        // additional calculation is necessary or if it indicates a potential
        // issue.
        let width = width + width_italic_buffer + offset as f32;

        // Create a new pixel map for the main text
        let mut img = tiny_skia::Pixmap::new(width as u32, height as u32)
            .ok_or(FontThumbnailError::FailedToCreatePixmap)?;
        // Draw the text into the pixel map
        buffer.draw(swash_cache, self.config.text_color, |x, y, w, h, color| {
            if let Some(rect) = tiny_skia::Rect::from_xywh(
                (x + offset) as f32,
                y as f32,
                w as f32,
                h as f32,
            )
            .map(Some)
            .unwrap_or_default()
            {
                img.fill_rect(
                    rect,
                    &get_skia_paint_for_color(color),
                    tiny_skia::Transform::identity(),
                    None,
                );
            } else {
                tracing::warn!(
                    "WARN: Failed to create rect: x: {x}, y: {y}, w: {w}, h: {h}"
                );
            }
        });

        // Create a pixel map for the total image
        let mut final_img = Pixmap::new(width as u32, height as u32)
            .ok_or(FontThumbnailError::FailedToCreatePixmap)?;
        // Fill the image in with black to start
        final_img.fill(self.config.background_color);
        // Draw the main text into the final image
        final_img.draw_pixmap(
            0,
            0,
            img.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            tiny_skia::Transform::identity(),
            None,
        );
        // Now use the `image` crate to save the final image as a PNG as
        // grayscale, because as of now, `tiny-skia` does not support
        // saving as PNG with grayscale
        // Convert the Pixmap to an RgbaImage
        let rgba_image = image::RgbaImage::from_raw(
            final_img.width(),
            final_img.height(),
            final_img.data().to_vec(),
        )
        .ok_or(FontThumbnailError::FailedToCreatePixmap)?;
        // Convert the RgbaImage to grayscale
        let gray_image =
            image::DynamicImage::ImageRgba8(rgba_image).grayscale();
        let mut png_buffer = Vec::new();
        let mut png_cursor = std::io::Cursor::new(&mut png_buffer);
        gray_image.write_to(&mut png_cursor, image::ImageFormat::Png)?;
        Ok(super::Thumbnail::new(
            png_buffer,
            Self::MIME_TYPE.to_string(),
        ))
    }
}

#[cfg(test)]
#[path = "png_thumbnail_test.rs"]
mod tests;
