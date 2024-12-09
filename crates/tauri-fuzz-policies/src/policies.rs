// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use crate::engine::{ConditionOnParameters, FuzzPolicy};
pub mod external_process;
pub mod filesystem;
mod utils;

#[cfg(unix)]
pub(crate) const LIBC: &str = "libc.";

pub fn no_policy() -> FuzzPolicy {
    vec![]
}

pub(crate) fn block_on_entry() -> ConditionOnParameters {
    std::sync::Arc::new(|_| Ok(true))
}

pub fn no_error_policy() -> FuzzPolicy {
    todo!()
}
