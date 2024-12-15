// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(unused_variables, unused_imports, dead_code)]
#![allow(
    clippy::semicolon_if_nothing_returned,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    clippy::missing_panics_doc,
    clippy::wildcard_imports
)]

mod utils;
use mini_app::basic::direct_panic;
use mini_app::file_access::read_foo_file;
use utils::*;

const BLOCKED_BINARY: &str = "dir";
const ARG: &str = "";

pub fn main() {
    color_backtrace::install();
    env_logger::init();

    fuzz_command_with_arg(
        "read_file",
        // Some(tauri_plugin_fs::commands::read_file::<tauri::test::MockRuntime> as usize),
        None,
        tauri_fuzz_policies::filesystem::no_file_access(),
        vec![("path", path_to_foo())],
        Some("fs".into()),
    );

    // fuzz_command_with_arg(
    //     "read_file",
    //     None,
    //     tauri_fuzz_policies::filesystem::write_only_access(),
    //     vec![("path", path_to_foo())],
    //     Some("fs".into()),
    // );

    // fuzz_command_with_arg(
    //     "ls_with_rust_command_output",
    //     Some(mini_app::external_process::ls_with_rust_command_output as usize),
    //     tauri_fuzz_policies::no_policy(),
    //     vec![("input", "-la")],
    //     None,
    // );
    //

    // fuzz_command_with_arg::<()>(
    //     "read_foo_file",
    //     Some(read_foo_file as usize),
    //     tauri_fuzz_policies::external_process::block_on_child_process_error_status(),
    //     // tauri_fuzz_policies::filesystem::write_only_access(),
    //     // tauri_fuzz_policies::no_policy(),
    //     // vec![("path", path_to_foo())],
    //     vec![],
    //     // Some("fs".into()),
    //     None,
    // );
}
