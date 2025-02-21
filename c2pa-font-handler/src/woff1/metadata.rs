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

//! WOFF 1.0 extension metadata support

use std::io::Write;

use serde::{Deserialize, Serialize};

use super::{
    directory::Woff1DirectoryEntry, font::Woff1Font, header::Woff1Header,
};
use crate::{data::Data, FontDirectory};

/// Metadata section of the WOFF file
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Metadata {
    version: String,
    unique_ids: Option<Vec<UniqueId>>,
    vendor: Option<Vec<Vendor>>,
    credits: Option<Vec<Credits>>,
    description: Option<Vec<Description>>,
    license: Option<Vec<License>>,
    copyright: Option<Vec<Copyright>>,
    trademark: Option<Vec<Trademark>>,
    licensee: Option<Vec<Licensee>>,
    extensions: Option<Vec<Extension>>,
}

/// Unique ID for the metadata
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct UniqueId {
    id: String,
}

/// Represents the text direction of text used in the metadata
#[derive(Debug, Default, Deserialize, Serialize)]
pub enum TextDirection {
    /// The text is written from left to right
    #[default]
    LeftToRight,
    /// The text is written from right to left
    RightToLeft,
}

/// Represents the vendor of the font
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Vendor {
    /// The name of the font vendor.
    name: String,
    /// The URL of the font vendor.
    url: Option<String>,
    /// The text direction.
    dir: Option<TextDirection>,
    /// An arbitrary set of tokens.
    class: Option<Vec<String>>,
}

/// A credit record, from the metadata
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Credit {
    /// The name of the entity being credited.
    name: String,
    /// The URL of the entity being credited.
    url: Option<String>,
    /// The role of the entity being credited.
    role: Option<String>,
    /// The text direction.
    dir: Option<TextDirection>,
    /// An arbitrary set of tokens.
    class: Option<Vec<String>>,
}

/// A list of credits, from the metadata
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Credits {
    credits: Vec<Credit>,
}

/// Represents text in the metadata
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Text {
    text: String,
    xml_lang: Option<String>,
    dir: Option<TextDirection>,
    class: Option<Vec<String>>,
}

/// An arbitrary text description of the font's design, history, etc.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Description {
    url: Option<String>,
    text: Vec<Text>,
}

/// Licensing information for the font
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct License {
    url: Option<String>,
    id: Option<String>,
    text: Vec<Text>,
}

/// The copyright for the font.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Copyright {
    text: Vec<Text>,
}

/// The trademark for the font.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Trademark {
    text: Vec<Text>,
}

/// The licensee for the font.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Licensee {
    name: String,
    dir: Option<TextDirection>,
    class: Option<Vec<String>>,
}

/// An extension item in the metadata.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ExtensionItem {
    id: Option<String>,
    name: Vec<Name>,
    value: Vec<Value>,
}

/// An extension to the metadata.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Extension {
    id: Option<String>,
    name: Option<Vec<String>>,
    item: Vec<ExtensionItem>,
}

/// A name in the metadata.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Name {
    xml_lang: Option<String>,
    dir: Option<TextDirection>,
    class: Option<Vec<String>>,
}

/// A value in the metadata.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Value {
    xml_lang: Option<String>,
    dir: Option<TextDirection>,
    class: Option<Vec<String>>,
}

/// Provides access to the `metadata` section of the WOFF file.
pub trait WoffMetadata {
    /// Returns the uncompressed metadata section, if any, of the WOFF file.
    fn metadata(&self) -> Option<Metadata>;

    /// Sets the metadata section of the WOFF file.
    fn set_metadata(&mut self, metadata: Metadata);
}

#[cfg(feature = "compression")]
impl WoffMetadata for Woff1Font {
    fn metadata(&self) -> Option<Metadata> {
        self.meta.as_ref().map(|t| {
            let mut output = Vec::new();
            let input_data = t.data();
            println!("length: {:?}", input_data.len());
            println!(" First 20 bytes: {:?}", &input_data[..20]);
            println!(
                " Last 20 bytes: {:?}",
                &input_data[input_data.len() - 20..]
            );
            let mut decoder = flate2::read::ZlibDecoder::new(input_data);
            std::io::copy(&mut decoder, &mut output)
                .expect("Failed to decompress metadata");

            serde_xml_rs::from_reader(&output[..])
                .expect("Failed to parse metadata")
        })
    }

    fn set_metadata(&mut self, metadata: Metadata) {
        let mut output = Vec::new();
        serde_xml_rs::to_writer(&mut output, &metadata)
            .expect("Failed to serialize metadata");
        let mut compressed = Vec::new();
        let mut encoder = flate2::write::ZlibEncoder::new(
            Vec::new(),
            flate2::Compression::default(),
        );
        encoder
            .write_all(&output)
            .expect("Failed to compress metadata");
        compressed = encoder.finish().expect("Failed to compress metadata");

        println!("length: {:?}", compressed.len());
        println!(" First 20 bytes: {:?}", &compressed[..20]);
        println!(" Last 20 bytes: {:?}", &compressed[compressed.len() - 20..]);
        self.meta = Some(Data::new(compressed));

        // Calculate the length of the metadata and making sure it is 4-byte
        // aligned
        let length = self.meta.as_ref().unwrap().data().len();
        let aligned_length = (length + 3) & !3;
        self.header.metaLength = aligned_length as u32;
        self.header.metaOrigLength = output.len() as u32;
        // Metadata offset is past the header, the directory, and all the tables
        // on a 4-byte boundary
        let metadata_offset = Woff1Header::SIZE as u32
            + self.directory.entries().len() as u32
                * Woff1DirectoryEntry::SIZE as u32
            + self
                .tables
                .values()
                .map(|t| (t.data().len() + 3) & !3)
                .sum::<usize>() as u32;
        self.header.metaOffset = metadata_offset;
    }
}

#[cfg(test)]
#[path = "metadata_test.rs"]
mod tests;
