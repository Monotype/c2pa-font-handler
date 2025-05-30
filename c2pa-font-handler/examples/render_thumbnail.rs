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

//! Example of generating a thumbnail for a font.

use c2pa_font_handler::thumbnail::{
    CosmicTextThumbnailGenerator, SvgThumbnailRenderer, ThumbnailGenerator,
};
use clap::{Parser, ValueEnum};

/// Types of thumbnails that can be generated.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ThumbnailType {
    /// A thumbnail in SVG format
    Svg,
    /// A thumbnail in PNG format
    Png,
}

/// An example of reading a WOFF file and writing information about it to the
/// console.
#[derive(Debug, Parser)]
struct Args {
    /// Input font file
    #[clap(short, long)]
    input: String,
    /// The output file path for saving the SVG thumbnail
    #[clap(short, long)]
    output: String,
    /// The type of thumbnail to generate
    #[clap(short, long, default_value = "svg")]
    thumbnail_type: ThumbnailType,
}

/// Main function for the render_thumbnail example.
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize the logger, can be controlled with RUST_LOG=debug,info,trace,
    // etc.
    tracing_subscriber::fmt::init();
    // Parse the command line arguments
    let args = Args::parse();

    let font_path = std::path::Path::new(&args.input);

    let generator: Box<dyn ThumbnailGenerator> = match args.thumbnail_type {
        ThumbnailType::Svg => {
            let renderer = Box::new(SvgThumbnailRenderer::default());
            Box::new(CosmicTextThumbnailGenerator::new(renderer))
        }
        ThumbnailType::Png => {
            return Err(anyhow::anyhow!(
                "PNG thumbnails are not supported yet"
            ));
        }
    };
    let thumbnail = generator
        .create_thumbnail(font_path)
        .map_err(|e| anyhow::anyhow!("Failed to create thumbnail: {}", e))?;
    println!(
        "Thumbnail created with mime type: {}",
        thumbnail.mime_type()
    );
    // Write the thumbnail to a file
    std::fs::write(&args.output, thumbnail.data()).map_err(|e| {
        anyhow::anyhow!("Failed to write thumbnail to file: {}", e)
    })?;
    println!("Thumbnail written to: {}", args.output);
    Ok(())
}
