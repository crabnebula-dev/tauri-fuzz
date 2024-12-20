// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(unused_variables, unused_imports)]

mod utils;
use mini_app::basic::direct_panic;
use mini_app::file_access::read_foo_file;
use utils::*;

pub fn main() {
    color_backtrace::install();
    env_logger::init();
    // fuzz_command_with_arg::<()>(
    //     "direct_panic",
    //     Some(direct_panic as usize),
    //     // tauri_fuzz_policies::filesystem::no_file_access(),
    //     // tauri_fuzz_policies::filesystem::write_only_access(),
    //     tauri_fuzz_policies::no_policy(),
    //     // vec![("path", path_to_foo())],
    //     vec![],
    //     // Some("fs".into()),
    //     None
    // );
    fuzz_command_with_arg::<()>(
        "read_foo_file",
        Some(read_foo_file as usize),
        tauri_fuzz_policies::filesystem::no_file_access(),
        // tauri_fuzz_policies::filesystem::write_only_access(),
        // tauri_fuzz_policies::no_policy(),
        // vec![("path", path_to_foo())],
        vec![],
        // Some("fs".into()),
        None,
    );
}
