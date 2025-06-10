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

//! Example of reading a WOFF1 font file.

use c2pa_font_handler::{
    c2pa::UpdatableC2PA, woff1::font::Woff1Font, Font, FontDataRead,
    FontDirectory, MutFontDataWrite,
};
use clap::Parser;

/// An example of reading a WOFF file and writing information about it to the
/// console.
#[derive(Debug, Parser)]
struct Args {
    /// Input font file
    #[clap(short, long)]
    input: String,

    /// Optional output file to write the font information to.
    #[clap(short, long)]
    output: Option<String>,

    /// Optional remote URL to associate with the font.
    #[clap(short, long)]
    remote_url: Option<String>,
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
    let mut font = Woff1Font::from_reader(&mut input_file)?;
    println!("{:#?}", font.header());
    for table in font.directory().entries() {
        println!("{table:#?}");
    }
    if let Some(output_file) = args.output {
        // Write the font information to the output file
        let mut output_file = std::fs::File::create(output_file)?;

        if let Some(remote_url) = &args.remote_url {
            let update_record = c2pa_font_handler::c2pa::UpdateContentCredentialRecord::builder()
            .with_active_manifest_uri(remote_url.to_string())
            .build();
            font.update_c2pa_record(update_record)?;
        }

        font.write(&mut output_file)?;
    }
    Ok(())
}
