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
