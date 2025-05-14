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

//! Profiler for benchmarking memory usage, using the `dhat` crate.

use criterion::profiler::Profiler;

/// Global allocator for profiling memory usage in benchmarks.
#[global_allocator]
pub static ALLOC: dhat::Alloc = dhat::Alloc;

/// A profiler to use with the dhat crate.
///
/// # Remarks
/// This profiler is intended to be used with the `criterion` crate for
/// benchmarking. It uses the `dhat` crate to profile memory usage during
/// benchmarks. To use it in your benchmarks, you need to set the `--profile`
/// flag when running the benchmarks. For example, to profile for 3 seconds
/// using the sfnt_c2pa_readwrite_mimic:
///
/// ```bash
/// cargo test --profile release --bench sfnt -- --profile-time 3 --bench sfnt_c2pa_readwrite_mimic
/// ```
///
/// And then to convert the output to a human-readable format, you can use the
/// `dhat-to-flamegraph` command line tool:
///
/// ```bash
/// # Install the dhat-to-flamegraph tool
/// cargo install dhat-to-flamegraph
/// # Convert the dhat output to a flamegraph
/// dhat-to-flamegraph  path/to/dhat-heap.json --output <output_file>
/// ```
pub struct DhatProfiler {
    _profiler: Option<dhat::Profiler>,
}

impl DhatProfiler {
    /// Creates a new [`DhatProfiler`].
    pub fn new() -> Self {
        Self { _profiler: None }
    }
}

// Implement the `Profiler` trait for `DhatProfiler`.
impl Profiler for DhatProfiler {
    fn start_profiling(
        &mut self,
        _benchmark_id: &str,
        _benchmark_dir: &std::path::Path,
    ) {
        self._profiler = Some(dhat::Profiler::new_heap());
    }

    fn stop_profiling(
        &mut self,
        _benchmark_id: &str,
        _benchmark_dir: &std::path::Path,
    ) {
        // Drop the profiler to write the report
        self._profiler = None;
    }
}
