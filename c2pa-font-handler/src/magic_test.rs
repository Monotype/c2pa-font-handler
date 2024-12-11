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

//! Tests for magic

use super::*;

#[test]
fn test_magic_from_u32() {
    let magic = Magic::try_from(0x00010000).unwrap();
    assert_eq!(magic as u32, Magic::TrueType as u32);

    let magic = Magic::try_from(0x4f54544f).unwrap();
    assert_eq!(magic as u32, Magic::OpenType as u32);

    let magic = Magic::try_from(0x74797031).unwrap();
    assert_eq!(magic as u32, Magic::PostScriptType1 as u32);

    let magic = Magic::try_from(0x74727565).unwrap();
    assert_eq!(magic as u32, Magic::AppleTrue as u32);

    let magic = Magic::try_from(0x774f4646).unwrap();
    assert_eq!(magic as u32, Magic::Woff as u32);

    let magic = Magic::try_from(0x774f4632).unwrap();
    assert_eq!(magic as u32, Magic::Woff2 as u32);
}

#[test]
fn test_magic_try_from_u32_with_bad_value() {
    let magic = Magic::try_from(0x00000000);
    assert!(magic.is_err());
    let err = magic.err().unwrap();
    assert!(matches!(err, FontIoError::UnknownMagic(0)));
}
