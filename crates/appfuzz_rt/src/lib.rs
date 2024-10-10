// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

mod fuzzer;
mod fuzzer_options;
mod runtime;
pub use crate::fuzzer::{fuzz_main, fuzz_test};
pub use crate::fuzzer_options::SimpleFuzzerConfig;

#[cfg(feature = "tauri")]
pub mod tauri;
