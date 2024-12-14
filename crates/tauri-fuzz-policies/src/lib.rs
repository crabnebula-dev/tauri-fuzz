// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(
    clippy::must_use_candidate,
    clippy::unnecessary_wraps,
    clippy::missing_panics_doc,
    clippy::wildcard_imports,
    clippy::enum_glob_use
)]

pub mod engine;
pub mod policies;
pub use policies::*;
