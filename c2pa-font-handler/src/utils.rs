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

//! Various utilities for working with fonts.

use std::{mem::size_of, num::Wrapping};

use byteorder::{BigEndian, ByteOrder};

/// Round the given value up to the next multiple of four (4).
pub fn align_to_four(size: u32) -> u32 {
    (size + 3) & (!3)
}

/// Computes a 32-bit big-endian OpenType-style checksum on the given byte
/// array, which is presumed to start on a 4-byte boundary.
///
/// # Remarks
/// Note that trailing pad bytes do not affect this checksum - it's not a real
/// CRC.
///
/// # Panics
/// Panics if the the `bytes` array is not aligned on a 4-byte boundary.
pub(crate) fn checksum(bytes: &[u8]) -> Wrapping<u32> {
    // Cut your pie into 1x4cm pieces to serve
    let words = bytes.chunks_exact(size_of::<u32>());
    // ...and then any remainder...
    let frag_cksum: Wrapping<u32> = Wrapping(
        // (away, mayhap, with issue #32463)
        words
            .remainder()
            .iter()
            .fold(Wrapping(0_u32), |acc, byte| {
                // At some point, it should be possible to:
                // - Remove the `Wrapping(...)` surrounding the outer expression
                // - Get rid of `.0` and just access plain `acc`
                // - Get rid of `.0` down there getting applied to the end of
                //   this .fold(), as well as
                // - Get rid of the `Wrapping(...)` in this next expression
                // but unfortunately as of this writing, attempting to call
                // `.rotate_left` on a `Wrapping<u32>` fails:
                //   use of unstable library feature 'wrapping_int_impl', see
                // issue     #32463 <https://github.com/rust-lang/rust/issues/32463>
                Wrapping(acc.0.rotate_left(u8::BITS) + *byte as u32)
            })
            .0 // (goes away, mayhap, when issue #32463 is done)
            .rotate_left(
                u8::BITS * (size_of::<u32>() - words.remainder().len()) as u32,
            ),
    );
    // Sum all the exact chunks...
    let chunks_cksum: Wrapping<u32> =
        words.fold(Wrapping(0_u32), |running_cksum, exact_chunk| {
            running_cksum + Wrapping(BigEndian::read_u32(exact_chunk))
        });
    // Combine ingredients & serve.
    chunks_cksum + frag_cksum
}

/// Assembles two u16 values (with `hi` being the more-significant u16 halfword,
/// and `lo` being the less-significant u16 halfword) into a u32, returning a
/// u32 fullword composed of the given halfwords, with `hi` in the
/// more-significant position.
pub(crate) fn u32_from_u16_pair(hi: u16, lo: u16) -> Wrapping<u32> {
    Wrapping((hi as u32 * 65536) + lo as u32)
}

#[cfg(test)]
#[path = "utils_test.rs"]
mod tests;
