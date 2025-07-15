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

//! Tests for MIME type handling in C2PA font handler.

use super::*;

#[test]
fn test_guess_mime_type_otf() {
    let mut reader = std::io::Cursor::new(&b"\x00\x01\x00\x00"[..]);
    let mime_type = reader.guess_mime_type().unwrap();
    assert_eq!(mime_type, &FontMimeTypes::OTF);

    let mut reader = std::io::Cursor::new(&b"OTTO"[..]);
    let mime_type = reader.guess_mime_type().unwrap();
    assert_eq!(mime_type, &FontMimeTypes::OTF);
}

#[test]
fn test_guess_mime_type_ttf() {
    let mut reader = std::io::Cursor::new(&b"true"[..]);
    let mime_type = reader.guess_mime_type().unwrap();
    assert_eq!(mime_type, &FontMimeTypes::TTF);
}

#[test]
fn test_guess_mime_type_woff() {
    let mut reader = std::io::Cursor::new(&b"\x77\x4F\x46\x46"[..]);
    let mime_type = reader.guess_mime_type().unwrap();
    assert_eq!(mime_type, &FontMimeTypes::WOFF);
}

#[test]
fn test_guess_mime_type_woff2() {
    let mut reader = std::io::Cursor::new(&b"\x77\x4F\x46\x32"[..]);
    let mime_type = reader.guess_mime_type().unwrap();
    assert_eq!(mime_type, &FontMimeTypes::WOFF2);
}

#[test]
fn test_guess_mime_type_unknown() {
    let mut reader = std::io::Cursor::new(&b"unknown"[..]);
    let result = reader.guess_mime_type();
    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap(),
        MimeTypeError::UnknownMagicType
    ));
}
