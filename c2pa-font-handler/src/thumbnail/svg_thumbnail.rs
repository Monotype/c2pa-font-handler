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

//! Thumbnail handling for C2PA fonts using SVG format.

use resvg::usvg::{Options, Tree};
use svg::{
    node::element::{Group, Style},
    Document, Node,
};

use super::{text::TextFontSystemContext, Renderer};
use crate::thumbnail::error::FontThumbnailError;

/// Trait for rounding values to a specified precision.
trait PrecisionRound {
    fn round_to(&self, precision: u32) -> Self;
}

// Implement PrecisionRound for f32
impl PrecisionRound for f32 {
    fn round_to(&self, precision: u32) -> Self {
        let factor = 10u32.pow(precision) as f32;
        (self * factor).round() / factor
    }
}

// Implement PrecisionRound for (f32, f32)
impl PrecisionRound for (f32, f32) {
    fn round_to(&self, precision: u32) -> Self {
        (self.0.round_to(precision), self.1.round_to(precision))
    }
}

/// Configuration for the SVG thumbnail renderer.
///
/// # Remarks
/// This configuration allows customization of the SVG thumbnail rendering,
/// including the precision of the coordinates and the fill color for the
/// glyphs.
///
/// Default values are provided for both precision and fill color, but they can
/// be overridden when creating an instance of this configuration.
#[derive(Clone, Debug)]
pub struct SvgThumbnailRendererConfig {
    /// The default precision for rounding SVG coordinates
    pub(crate) default_precision: u32,
    /// The fill color for the glyphs in the SVG thumbnail
    pub(crate) glyph_fill_color: String,
}

impl SvgThumbnailRendererConfig {
    /// Default precision for SVG values.
    pub const DEFAULT_SVG_PRECISION: u32 = 2;
    /// Default fill color for glyphs in SVG thumbnails.
    pub const SVG_GLYPH_FILL_COLOR: &'static str = "black";

    /// Create a new SVG thumbnail renderer configuration.
    pub fn new<S: Into<String>>(
        default_precision: u32,
        glyph_fill_color: S,
    ) -> Self {
        Self {
            default_precision,
            glyph_fill_color: glyph_fill_color.into(),
        }
    }
}

impl Default for SvgThumbnailRendererConfig {
    fn default() -> Self {
        Self::new(
            SvgThumbnailRendererConfig::DEFAULT_SVG_PRECISION,
            SvgThumbnailRendererConfig::SVG_GLYPH_FILL_COLOR.to_string(),
        )
    }
}

/// Renderer for SVG thumbnails from font data.
pub struct SvgThumbnailRenderer {
    /// Configuration for the SVG thumbnail renderer.
    config: SvgThumbnailRendererConfig,
}

impl SvgThumbnailRenderer {
    /// The name of the SVG fill attribute.
    const FILL: &'static str = "fill";
    /// The MIME type for SVG thumbnails.
    const MIME_TYPE: &'static str = "image/svg+xml";
    /// The name of the SVG path element.
    const PATH: &'static str = "path";
    /// The scale transformation to flip the SVG vertically.
    const SCALE: &'static str = "scale(1, -1)";
    /// The name of the SVG transform attribute.
    const TRANSFORM: &'static str = "transform";
    /// The viewBox attribute for the SVG document.
    const VIEW_BOX: &'static str = "viewBox";

    /// Create a new SVG thumbnail renderer with the given configuration.
    pub fn new(config: SvgThumbnailRendererConfig) -> Self {
        Self { config }
    }
}

impl Default for SvgThumbnailRenderer {
    fn default() -> Self {
        Self::new(SvgThumbnailRendererConfig::default())
    }
}

impl Renderer for SvgThumbnailRenderer {
    fn render_thumbnail(
        &self,
        text_system_context: &mut TextFontSystemContext,
    ) -> Result<super::Thumbnail, super::error::FontThumbnailError> {
        let precision = self.config.default_precision;
        tracing::trace!("Rendering SVG thumbnail with precision: {precision}");
        let mut svg_doc = Document::new();
        let mut tmp_doc = Document::new();
        let (font_system, swash_cache, text_buffer) =
            text_system_context.mut_cosmic_text_parts();
        for layout_run in text_buffer.layout_runs() {
            let mut group = Group::new();
            // Add a style to have the fill as black and the stroke to none
            group = group.add(Style::new(
                format!(
                    "{} {{ {}: {}; }}",
                    Self::PATH,
                    Self::FILL,
                    self.config.glyph_fill_color
                )
                .as_str(),
            ));
            for glyph in layout_run.glyphs {
                let mut data = svg::node::element::path::Data::new();
                // Get the x/y offsets
                let (x_offset, y_offset) =
                    (glyph.x + glyph.x_offset, glyph.y + glyph.y_offset);
                // We will need the physical glyph to get the outline commands
                let physical_glyph = glyph.physical((0., 0.), 1.0);
                let cache_key = physical_glyph.cache_key;
                let outline_commands =
                    swash_cache.get_outline_commands(font_system, cache_key);
                // Go through each command and build the path
                if let Some(commands) = outline_commands {
                    for command in commands {
                        match command {
                            cosmic_text::Command::MoveTo(p1) => {
                                let rounded_data =
                                    (p1.x, p1.y).round_to(precision);
                                data = data.move_to(rounded_data);
                            }
                            cosmic_text::Command::LineTo(p1) => {
                                let rounded_data =
                                    (p1.x, p1.y).round_to(precision);
                                data = data.line_to(rounded_data);
                            }
                            cosmic_text::Command::CurveTo(p1, p2, p3) => {
                                let p1_rounded_data =
                                    (p1.x, p1.y).round_to(precision);
                                let p2_rounded_data =
                                    (p2.x, p2.y).round_to(precision);
                                let p3_rounded_data =
                                    (p3.x, p3.y).round_to(precision);
                                data = data.cubic_curve_to((
                                    p1_rounded_data,
                                    p2_rounded_data,
                                    p3_rounded_data,
                                ));
                            }
                            cosmic_text::Command::QuadTo(p1, p2) => {
                                let p1_rounded_data =
                                    (p1.x, p1.y).round_to(precision);
                                let p2_rounded_data =
                                    (p2.x, p2.y).round_to(precision);
                                data = data.quadratic_curve_to((
                                    p1_rounded_data,
                                    p2_rounded_data,
                                ));
                            }
                            cosmic_text::Command::Close => {
                                data = data.close();
                            }
                        }
                    }
                }
                // Don't add empty data paths
                if !data.is_empty() {
                    let path = svg::node::element::Path::new()
                        .set(
                            Self::TRANSFORM,
                            format!("translate({x_offset}, {y_offset})"),
                        )
                        .set("d", data.clone());
                    group = group.add(path);
                }
            }

            group.assign(Self::TRANSFORM, Self::SCALE);
            // We will need to create a temporary document to get the bounding
            // box of the entire group
            tmp_doc = tmp_doc.add(group.clone());
            svg_doc.append(group);
        }
        // Convert the temporary document to a string, so we can get the
        // bounding box
        let svg_str = tmp_doc.to_string();
        // Generate the SVG tree from the string
        let tree =
            Tree::from_str(&svg_str, &Options::default()).map_err(|e| {
                tracing::trace!(
                    "Failed to create SVG tree from string: {svg_str}"
                );
                tracing::error!("Failed to create SVG tree with error: {e}");
                FontThumbnailError::FailedToCreateSvgTree(svg_str)
            })?;
        // Round the bounding box outwards and then convert it to a rect
        let bounding_box = tree
            .root()
            .abs_bounding_box()
            .round_out()
            .ok_or(FontThumbnailError::InvalidRect)?
            .to_rect();
        // Set the view box with a 1-pixel padding around the bounding box, as
        // there are some corner cases (when the font is bold?) where
        // the bounding box is not quite right and 1 pixel row is being
        // clipped for items where the character goes below the
        // baseline.
        svg_doc = svg_doc.set(
            Self::VIEW_BOX,
            (
                bounding_box.x() - 1.0,
                bounding_box.y() - 1.0,
                bounding_box.width() + 2.0,
                bounding_box.height() + 2.0,
            ),
        );

        let mut svg_buffer = Vec::new();
        let svg_cursor = std::io::Cursor::new(&mut svg_buffer);
        svg::write(svg_cursor, &svg_doc)?;
        Ok(super::Thumbnail::new(
            svg_buffer,
            SvgThumbnailRenderer::MIME_TYPE.to_string(),
        ))
    }
}

#[cfg(test)]
#[path = "svg_thumbnail_test.rs"]
mod tests;
