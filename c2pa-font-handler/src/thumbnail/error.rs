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

//! Thumbnail error types.

/// Errors that can occur when creating a font thumbnail
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FontThumbnailError {
    /*
    /// Error from the image crate
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
    */
    /// error from IO operations
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    /// A font was not found
    #[error("No font found")]
    NoFontFound,
    /// No full name found in the font
    #[error("No full name found")]
    NoFullNameFound,
    /// Error when the buffer size is invalid
    #[error("The buffer size is invalid")]
    InvalidBufferSize,
    #[cfg(feature = "svg-thumbnails")]
    /// Could not create a Rect from the given values
    #[error("Invalid values for Rect")]
    InvalidRect,
    /// Failed to find a point size that would accommodate the width and text
    #[error("Failed to find an appropriate font point size to fit the width")]
    FailedToFindAppropriateSize,
    /// Failed to create a Pixmap object
    #[error("Failed to create pixmap")]
    FailedToCreatePixmap,
    /// Failed to get a pixel from the image
    #[error("Failed to get pixel from image; x: {x}, y: {y}")]
    FailedToGetPixel { x: u32, y: u32 },
    /// Failed to create a SVG tree from string
    #[cfg(feature = "svg-thumbnails")]
    #[error("Failed to create a SVG tree from string: {0}")]
    FailedToCreateSvgTree(String),
    #[cfg(not(feature = "svg-thumbnails"))]
    /// The SVG feature is not enabled
    #[error("The SVG feature is not enabled")]
    SvgFeatureNotEnabled,
}
