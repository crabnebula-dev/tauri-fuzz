// // Copyright 2024-2022 CrabNebula Ltd.
// // SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(unused_variables)]
mod utils;
use utils::*;

pub fn main() {
    color_backtrace::install();
    env_logger::init();
    fuzz_command_with_arg(
        "read_file",
        None,
        policies::filesystem::no_file_access(),
        // policies::filesystem::write_only_access(),
        // policies::no_policy(),
        vec![("path", path_to_foo())],
        Some("fs".into()),
    );
}
