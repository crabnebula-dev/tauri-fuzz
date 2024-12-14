// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::default_trait_access,
    clippy::semicolon_if_nothing_returned,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::manual_assert,
    clippy::unnecessary_wraps,
    clippy::explicit_iter_loop,
    clippy::needless_pass_by_value,
    clippy::doc_markdown
)]
mod fuzzer;
mod fuzzer_options;
mod runtime;
pub use crate::fuzzer::{fuzz_main, fuzz_test};
pub use crate::fuzzer_options::SimpleFuzzerConfig;

#[cfg(feature = "tauri")]
pub mod tauri;
