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

//! Various utilities for C2PA table data for benchmarks.

use std::sync::OnceLock;

use c2pa_font_handler::{sfnt::table::TableC2PA, FontDataExactRead};

/// Static lock around the faux C2PA table.
pub static SFNT_FAUX_C2PA_TABLE: OnceLock<TableC2PA> = OnceLock::new();

/// Gets data for a faux C2PA table from the file system.
pub fn get_faux_c2pa_table_data() -> (&'static [u8], usize) {
    let data = include_bytes!("../../../.devtools/faux_c2pa_table.bin");
    let length = data.len();
    (data, length)
}

/// Gets the faux C2PA table from the file system.
pub fn get_faux_c2pa_table() -> &'static TableC2PA {
    SFNT_FAUX_C2PA_TABLE.get_or_init(|| {
        let (data, length) = get_faux_c2pa_table_data();
        let mut font_stream = std::io::Cursor::new(data);
        TableC2PA::from_reader_exact(&mut font_stream, 0, length)
            .expect("Failed to read faux C2PA table")
    })
}
