// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! Module with utilities functions to fuzz Tauri applications

mod from_random_bytes;
mod macros;
mod test_helpers;
mod utils;

pub use from_random_bytes::FromRandomBytes;
pub use test_helpers::*;
pub use utils::*;
