// Copyright 2024 Monotype Imaging Inc.
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

//! Example of using the font-io library to stub a DSIG table in an SFNT
//! font.

use c2pa_font_handler::{FontDSIGStubber, FontDataRead, MutFontDataWrite};
use clap::Parser;

/// This tool can be used to take a font file and remove the DSIG table from it,
/// or replace it with a stub table.
#[derive(Debug, Parser)]
struct Args {
    /// Input font file
    #[clap(short, long)]
    input: String,
    /// Output font file
    #[clap(short, long)]
    output: String,
}

/// Main function for the stub_dsig example.
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize the logger, can be controlled with RUST_LOG=debug,info,trace,
    // etc.
    tracing_subscriber::fmt::init();
    // Parse the command line arguments
    let args = Args::parse();

    // Open the input file
    let mut input_file = std::fs::File::open(&args.input)?;
    // Read the font file
    let mut font =
        c2pa_font_handler::sfnt::font::SfntFont::from_reader(&mut input_file)?;
    // Stub the DSIG table
    font.stub_dsig()?;
    // Open the output file
    let mut output_file = std::fs::File::create(&args.output)?;
    // And write the font file
    font.write(&mut output_file)?;

    Ok(())
}
