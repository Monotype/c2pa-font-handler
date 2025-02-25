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
use crate::{data::Data, error::FontIoError, FontDirectory};

/// Metadata section of the WOFF file
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename = "metadata")]
pub struct Metadata {
    #[serde(rename = "@version")]
    version: String,
    #[serde(rename = "uniqueid", skip_serializing_if = "Option::is_none")]
    unique_ids: Option<Vec<UniqueId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vendor: Option<Vec<Vendor>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    credits: Option<Credits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Vec<Description>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<Vec<License>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    copyright: Option<Vec<Copyright>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trademark: Option<Vec<Trademark>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    licensee: Option<Vec<Licensee>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extensions: Option<Vec<Extension>>,
}

/// Unique ID for the metadata
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct UniqueId {
    #[serde(rename = "@id")]
    id: String,
}

/// Represents the text direction of text used in the metadata
#[derive(Debug, Default, Deserialize, Serialize)]
pub enum TextDirection {
    /// The text is written from left to right
    #[default]
    #[serde(rename = "ltr")]
    LeftToRight,
    /// The text is written from right to left
    #[serde(rename = "rtl")]
    RightToLeft,
}

/// Represents the vendor of the font
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Vendor {
    /// The name of the font vendor.
    #[serde(rename = "@name")]
    name: String,
    /// The URL of the font vendor.
    #[serde(rename = "@url", skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    /// The text direction.
    #[serde(rename = "@dir", skip_serializing_if = "Option::is_none")]
    dir: Option<TextDirection>,
    /// An arbitrary set of tokens.
    #[serde(rename = "@class", skip_serializing_if = "Option::is_none")]
    class: Option<Vec<String>>,
}

/// A credit record, from the metadata
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Credit {
    /// The name of the entity being credited.
    #[serde(rename = "@name")]
    name: String,
    /// The URL of the entity being credited.
    #[serde(rename = "@url", skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    /// The role of the entity being credited.
    #[serde(rename = "@role", skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    /// The text direction.
    #[serde(rename = "@dir", skip_serializing_if = "Option::is_none")]
    dir: Option<TextDirection>,
    /// An arbitrary set of tokens.
    #[serde(rename = "@class", skip_serializing_if = "Option::is_none")]
    class: Option<Vec<String>>,
}

/// A list of credits, from the metadata
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Credits {
    #[serde(rename = "credit")]
    credits: Vec<Credit>,
}

/// Represents text in the metadata
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Text {
    #[serde(rename = "$value")]
    text: String,
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    xml_lang: Option<String>,
    #[serde(rename = "@dir", skip_serializing_if = "Option::is_none")]
    dir: Option<TextDirection>,
    #[serde(rename = "@class", skip_serializing_if = "Option::is_none")]
    class: Option<Vec<String>>,
}

/// An arbitrary text description of the font's design, history, etc.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Description {
    #[serde(rename = "@url", skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    text: Vec<Text>,
}

/// Licensing information for the font
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct License {
    #[serde(rename = "@url", skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
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
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@dir", skip_serializing_if = "Option::is_none")]
    dir: Option<TextDirection>,
    #[serde(rename = "@class", skip_serializing_if = "Option::is_none")]
    class: Option<Vec<String>>,
}

/// An extension item in the metadata.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ExtensionItem {
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(rename = "name")]
    name: Vec<Name>,
    #[serde(rename = "$value")]
    value: Vec<Value>,
}

/// An extension to the metadata.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Extension {
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    name: Vec<String>,
    item: Vec<ExtensionItem>,
}

/// A name in the metadata.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Name {
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    xml_lang: Option<String>,
    #[serde(rename = "@dir", skip_serializing_if = "Option::is_none")]
    dir: Option<TextDirection>,
    #[serde(rename = "@class", skip_serializing_if = "Option::is_none")]
    class: Option<Vec<String>>,
}

/// A value in the metadata.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Value {
    #[serde(rename = "@xml:lang", skip_serializing_if = "Option::is_none")]
    xml_lang: Option<String>,
    #[serde(rename = "@dir", skip_serializing_if = "Option::is_none")]
    dir: Option<TextDirection>,
    #[serde(rename = "@class", skip_serializing_if = "Option::is_none")]
    class: Option<Vec<String>>,
    #[serde(rename = "$value", skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

/// Provides access to the `metadata` section of the WOFF file.
pub trait WoffMetadata {
    /// The error type for this trait.
    type Error;
    /// Returns the uncompressed metadata section, if any, of the WOFF file.
    fn metadata(&self) -> Result<Option<Metadata>, Self::Error>;

    /// Sets the metadata section of the WOFF file.
    fn set_metadata(&mut self, metadata: Metadata) -> Result<(), Self::Error>;
}

#[cfg(feature = "compression")]
impl WoffMetadata for Woff1Font {
    type Error = FontIoError;

    fn metadata(&self) -> Result<Option<Metadata>, Self::Error> {
        if let Some(meta) = self.meta.as_ref() {
            let mut output = Vec::new();
            let input_data = meta.data();
            let mut decoder = flate2::read::ZlibDecoder::new(input_data);
            std::io::copy(&mut decoder, &mut output)?;
            Ok(quick_xml::de::from_reader(&output[..])?)
        } else {
            Ok(None)
        }
    }

    fn set_metadata(&mut self, metadata: Metadata) -> Result<(), Self::Error> {
        let mut output = String::new();
        quick_xml::se::to_writer(&mut output, &metadata)?;
        let mut encoder = flate2::write::ZlibEncoder::new(
            Vec::new(),
            flate2::Compression::default(),
        );
        println!("Data: {:#?}", output);
        encoder.write_all(output.as_bytes())?;
        let compressed = encoder.finish()?;

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
        Ok(())
    }
}

#[cfg(test)]
#[path = "metadata_test.rs"]
mod tests;
