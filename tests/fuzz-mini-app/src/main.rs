// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(unused_variables, unused_imports)]

mod utils;
use mini_app::basic::direct_panic;
use mini_app::file_access::read_foo_file;
use utils::*;

const BLOCKED_BINARY: &str = "cmd";
const ARG: &str = "";

pub fn main() {
    color_backtrace::install();
    env_logger::init();
    // let mut command = std::process::Command::new("cmd");
    // command.args(&["/C", "dir", ""]);
    // let output = command.output();
    // println!("{:?}", output);

    fuzz_command_with_arg(
        "ls_with_rust_command_output",
        Some(mini_app::external_process::ls_with_rust_command_output as usize),
        tauri_fuzz_policies::external_process::block_on_child_process_error_status(),
        vec![("input", "sdfjkl")],
        None,
    )

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
