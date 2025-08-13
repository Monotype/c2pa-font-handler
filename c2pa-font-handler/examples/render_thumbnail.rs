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
    CosmicTextThumbnailGenerator, PngThumbnailRenderer, SvgThumbnailRenderer,
    ThumbnailGenerator,
};
use clap::{Parser, ValueEnum};
use tracing_subscriber::{
    fmt::{format::FmtSpan, writer::BoxMakeWriter},
    EnvFilter,
};

/// Default font size used when the fixed search strategy is selected.
const DEFAULT_FIXED_FONT_SIZE: f32 = 32.0;

/// Types of thumbnails that can be generated.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ThumbnailType {
    /// A thumbnail in SVG format
    Svg,
    /// A thumbnail in PNG format
    Png,
}

/// Strategies for searching for the appropriate font size.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum SearchStrategy {
    /// A linear search strategy for font sizes
    ///
    /// # Remarks
    /// A default linear configuration will be used.
    Linear,
    /// A binary search strategy for font sizes
    ///
    /// # Remarks
    /// A default binary configuration will be used.
    Binary,
    /// A fixed font size; this will use the default (32.0) fixed font size
    Fixed,
}

// Allow for conversion from `SearchStrategy` to the internal
// `FontSizeSearchStrategy` used by the thumbnail generator.
impl From<SearchStrategy>
    for c2pa_font_handler::thumbnail::FontSizeSearchStrategy
{
    fn from(strategy: SearchStrategy) -> Self {
        match strategy {
            SearchStrategy::Linear => {
                c2pa_font_handler::thumbnail::FontSizeSearchStrategy::Linear(
                    c2pa_font_handler::thumbnail::LinearSearchContext::default(
                    ),
                )
            }
            SearchStrategy::Binary => {
                c2pa_font_handler::thumbnail::FontSizeSearchStrategy::Binary(
                    c2pa_font_handler::thumbnail::BinarySearchContext::default(
                    ),
                )
            }
            SearchStrategy::Fixed => {
                c2pa_font_handler::thumbnail::FontSizeSearchStrategy::Fixed(
                    DEFAULT_FIXED_FONT_SIZE,
                )
            }
        }
    }
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
    /// Optional log file to write logs to. If not provided, logs will be
    /// written to stdout.
    #[clap(long, default_value = None)]
    log_file: Option<String>,
    /// The search strategy for finding the appropriate font size
    #[clap(long, default_value = "binary")]
    search_strategy: SearchStrategy,
}

/// Destination for logging output.
enum LogDestination {
    /// Log to a file
    File(std::path::PathBuf),
    /// Log to stdout
    Stdout,
}

impl LogDestination {
    /// Create a writer for the log destination, used with the tracing
    /// subscriber.
    fn make_writer(&self) -> BoxMakeWriter {
        match self {
            LogDestination::File(file) => {
                let file = std::fs::File::create(file)
                    .expect("Failed to create log file");
                BoxMakeWriter::new(move || file.try_clone().unwrap())
            }
            LogDestination::Stdout => BoxMakeWriter::new(std::io::stdout),
        }
    }
}

/// Initialize the logger with the specified log file or stdout.
fn init_logger(log_file: Option<String>) {
    // Initialize the logger, can be controlled with RUST_LOG=debug,info,trace,
    // etc.
    let log_stream = if let Some(log_file) = log_file {
        LogDestination::File(log_file.into())
    } else {
        LogDestination::Stdout
    };
    let writer = log_stream.make_writer();
    tracing_subscriber::fmt::SubscriberBuilder::default()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .with_current_span(true)
        .with_writer(writer)
        .init();
}

/// Main function for the render_thumbnail example.
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Parse the command line arguments
    let args = Args::parse();
    // Initialize the logger
    init_logger(args.log_file);

    let font_path = std::path::Path::new(&args.input);

    let generator: Box<dyn ThumbnailGenerator> = match args.thumbnail_type {
        ThumbnailType::Svg => {
            let renderer = Box::new(SvgThumbnailRenderer::default());
            Box::new(CosmicTextThumbnailGenerator::new_with_config(
                renderer,
                c2pa_font_handler::thumbnail::FontSystemConfig::builder()
                    .search_strategy(args.search_strategy.into())
                    .build(),
            ))
        }
        ThumbnailType::Png => {
            let renderer = Box::new(PngThumbnailRenderer::default());
            Box::new(CosmicTextThumbnailGenerator::new(renderer))
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
