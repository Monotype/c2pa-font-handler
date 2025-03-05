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

//! Various font Magic bytes.

use super::error::FontIoError;

/// 32-bit font-format identification magic number.
///
/// Note that Embedded OpenType and MicroType Express formats cannot be detected
/// with a simple magic-number sniff. Conceivably, EOT could be dealt with as a
/// variation on SFNT, but MTX will needs more exotic handling.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Magic {
    /// 'OTTO' - OpenType
    OpenType = 0x4f54544f,
    /// FIXED 1.0 - TrueType (or possibly v1.0 Embedded OpenType)
    TrueType = 0x00010000,
    /// 'typ1' - PostScript Type 1
    PostScriptType1 = 0x74797031,
    /// 'true' - TrueType fonts for OS X / iOS
    AppleTrue = 0x74727565,
    /// 'wOFF' - WOFF 1.0
    Woff = 0x774f4646,
    /// 'wOF2' - WOFF 2.0
    Woff2 = 0x774f4632,
}

/// Used to attempt conversion from u32 to a Magic value.
impl TryFrom<u32> for Magic {
    type Error = FontIoError;

    /// Try to match the given u32 value to a known font-format magic number.
    fn try_from(v: u32) -> core::result::Result<Self, Self::Error> {
        match v {
            ot if ot == Magic::OpenType as u32 => Ok(Magic::OpenType),
            tt if tt == Magic::TrueType as u32 => Ok(Magic::TrueType),
            t1 if t1 == Magic::PostScriptType1 as u32 => {
                Ok(Magic::PostScriptType1)
            }
            at if at == Magic::AppleTrue as u32 => Ok(Magic::AppleTrue),
            w1 if w1 == Magic::Woff as u32 => Ok(Magic::Woff),
            w2 if w2 == Magic::Woff2 as u32 => Ok(Magic::Woff2),
            _unknown => Err(FontIoError::UnknownMagic(v)),
        }
    }
}

#[cfg(test)]
#[path = "magic_test.rs"]
mod tests;
