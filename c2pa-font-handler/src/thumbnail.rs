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

//! Thumbnail handling for C2PA fonts.

use std::{
    fs::File,
    io::{Read, Seek},
};

pub(crate) mod error;
#[cfg(feature = "svg-thumbnails")]
pub(crate) mod svg_thumbnail;
#[cfg(feature = "svg-thumbnails")]
pub use svg_thumbnail::{SvgThumbnailRenderer, SvgThumbnailRendererConfig};

pub(crate) mod text;
pub use text::{CosmicTextThumbnailGenerator, FontSystemConfig};

/// Represents a thumbnail.
pub struct Thumbnail {
    /// The raw data of the thumbnail.
    pub(crate) data: Vec<u8>,
    /// The mime type of the thumbnail.
    pub(crate) mime_type: String,
}

impl Thumbnail {
    /// Create a new thumbnail with the given data and mime type.
    #[allow(dead_code)]
    fn new(data: Vec<u8>, mime_type: String) -> Self {
        Self { data, mime_type }
    }

    /// Get the data of the thumbnail.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the mime type of the thumbnail.
    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }
}

/// Trait for rendering thumbnails.
#[cfg_attr(test, mockall::automock)]
pub trait Renderer {
    /// Render the thumbnail to a writer.
    ///
    /// # Parameters
    /// - `writer`: A mutable reference to a writer that implements `Write`.
    ///
    /// # Errors
    /// Returns an error if the thumbnail could not be rendered.
    fn render_thumbnail(
        &self,
        text_buffer: &mut cosmic_text::Buffer,
        font_system: &mut cosmic_text::FontSystem,
        swash_cache: &mut cosmic_text::SwashCache,
    ) -> Result<Thumbnail, error::FontThumbnailError>;
}

/// Marker trait for types that can read and seek.
pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek + ?Sized> ReadSeek for T {}

/// Support for creating thumbnails for font software.
pub trait ThumbnailGenerator {
    /// Create a thumbnail for the font.
    ///
    /// This function will create a thumbnail for the font, which can be used
    /// in C2PA operations.
    ///
    /// # Parameters
    /// - `path`: The path to the font file for which the thumbnail should be
    ///   created.
    ///
    /// # Errors
    /// Returns an error if the thumbnail could not be created.
    ///
    /// # Remarks
    /// The default implementation will use [`std::fs::File`] to read the
    /// font file.
    fn create_thumbnail(
        &self,
        path: &std::path::Path,
    ) -> Result<Thumbnail, error::FontThumbnailError> {
        let mut reader =
            File::open(path).map_err(error::FontThumbnailError::IoError)?;
        self.create_thumbnail_from_stream(&mut reader)
    }

    /// Create a thumbnail from a stream.
    ///
    /// This function will create a thumbnail for the font from a reader stream,
    /// which can be used in C2PA operations.
    ///
    /// # Parameters
    /// - `reader`: A mutable reference to a reader that implements `Read` and
    ///   `Seek`.
    ///
    /// # Errors
    /// Returns an error if the thumbnail could not be created from the stream.
    fn create_thumbnail_from_stream(
        &self,
        reader: &mut (dyn ReadSeek),
    ) -> Result<Thumbnail, error::FontThumbnailError>;
}
