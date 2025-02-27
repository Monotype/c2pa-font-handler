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

//! Tests for utils

use super::*;

#[test]
fn test_u32_from_u16_pair() {
    let (u16_hi, u16_low) = (0x1234, 0x5678);
    let u32 = u32_from_u16_pair(u16_hi, u16_low);
    assert_eq!(u32, Wrapping(0x12345678));
}

#[test]
fn test_align_to_four() {
    let size = 5;
    let aligned_size = align_to_four(size);
    assert_eq!(aligned_size, 8);
}

#[test]
fn test_checksum() {
    let data = [0x00, 0x01, 0x02, 0x03];
    let checksum = checksum(&data);
    assert_eq!(checksum, Wrapping(0x00010203));
}

/// Verifies the adding of a remote C2PA manifest reference works as
/// expected.
#[test]
fn test_checksum_and_biased() {
    let data = [
        0x0f, 0x0f, 0x0f, 0x0f, 0x04, 0x03, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01, 0x01, 0x00,
    ];
    let expected: [[u32; 17]; 4] = [
        [
            0x00000000_u32,
            0x0f000000_u32,
            0x0f0f0000_u32,
            0x0f0f0f00_u32,
            0x0f0f0f0f_u32,
            0x130f0f0f_u32,
            0x13120f0f_u32,
            0x1312110f_u32,
            0x13121110_u32,
            0x13121110_u32,
            0x13121110_u32,
            0x13121110_u32,
            0x13121110_u32,
            0x13121110_u32,
            0x13131110_u32,
            0x13131210_u32,
            0x13131210_u32,
        ],
        [
            0x00000000_u32,
            0x000f0000_u32,
            0x000f0f00_u32,
            0x000f0f0f_u32,
            0x0f0f0f0f_u32,
            0x0f130f0f_u32,
            0x0f13120f_u32,
            0x0f131211_u32,
            0x10131211_u32,
            0x10131211_u32,
            0x10131211_u32,
            0x10131211_u32,
            0x10131211_u32,
            0x10131211_u32,
            0x10131311_u32,
            0x10131312_u32,
            0x10131312_u32,
        ],
        [
            0x00000000_u32,
            0x00000f00_u32,
            0x00000f0f_u32,
            0x0f000f0f_u32,
            0x0f0f0f0f_u32,
            0x0f0f130f_u32,
            0x0f0f1312_u32,
            0x110f1312_u32,
            0x11101312_u32,
            0x11101312_u32,
            0x11101312_u32,
            0x11101312_u32,
            0x11101312_u32,
            0x11101312_u32,
            0x11101313_u32,
            0x12101313_u32,
            0x12101313_u32,
        ],
        [
            0x00000000_u32,
            0x0000000f_u32,
            0x0f00000f_u32,
            0x0f0f000f_u32,
            0x0f0f0f0f_u32,
            0x0f0f0f13_u32,
            0x120f0f13_u32,
            0x12110f13_u32,
            0x12111013_u32,
            0x12111013_u32,
            0x12111013_u32,
            0x12111013_u32,
            0x12111013_u32,
            0x12111013_u32,
            0x13111013_u32,
            0x13121013_u32,
            0x13121013_u32,
        ],
    ];
    for frag_length in 0..data.len() {
        // Create a fragment from the first N bytes
        let frag_0n = &data[0..frag_length];
        // Verify its checksum for different bias values
        let cksum = checksum(frag_0n);
        let cksum_0 = checksum_biased(frag_0n, 0);
        let cksum_1 = checksum_biased(frag_0n, 1);
        let cksum_2 = checksum_biased(frag_0n, 2);
        let cksum_3 = checksum_biased(frag_0n, 3);
        assert_eq!(expected[0][frag_length], cksum.0);
        assert_eq!(expected[0][frag_length], cksum_0.0);
        assert_eq!(expected[1][frag_length], cksum_1.0);
        assert_eq!(expected[2][frag_length], cksum_2.0);
        assert_eq!(expected[3][frag_length], cksum_3.0);
    }
}
