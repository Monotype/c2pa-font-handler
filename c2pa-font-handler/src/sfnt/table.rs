// Copyright 2024-2025 Monotype Imaging Inc.
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

//! SFNT font file table.

pub(crate) mod c2pa;
pub(crate) mod dsig;
pub(crate) mod head;
pub(crate) mod named_table;

// Export C2PA table
pub use c2pa::TableC2PA;
// Export DSIG table
pub use dsig::TableDSIG;
// Export head table
pub use head::TableHead;
// Export named table
pub use named_table::NamedTable;
