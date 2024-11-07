// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use fuzz_mini_app::utils::fuzz_command_with_arg;
use tauri_fuzz::tauri::{start_crashing_fuzz_process, start_non_crashing_fuzz_process};

// This is a trick to test fuzzers with multi-threaded and get fuzzer output when crashing.
// Frida-gum does not support multi-threads therefore we start fuzzing in different processes.
// The "hidden_*"  test will be started in a separate process and the exit status will be captured
// by the parent process/test.
#[test]
fn block_read_foo() {
    start_crashing_fuzz_process("hidden_block_read_foo_with_nofileaccess_policy");
    start_crashing_fuzz_process("hidden_block_read_foo_with_filename_policy");
}

// Read-only access should not block `read_foo`
#[test]
fn allow_read_foo() {
    start_non_crashing_fuzz_process("hidden_allow_read_foo_with_no_policy");
    start_non_crashing_fuzz_process("hidden_allow_read_foo_with_readonly_policy");
}

#[test]
#[ignore]
fn hidden_block_read_foo_with_nofileaccess_policy() {
    fuzz_command_with_arg(
        "read_foo_file",
        Some(mini_app::file_access::read_foo_file as usize),
        tauri_fuzz_policies::filesystem::no_file_access(),
        Vec::<(&str, ())>::new(),
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_read_foo_with_filename_policy() {
    fuzz_command_with_arg(
        "read_foo_file",
        Some(mini_app::file_access::read_foo_file as usize),
        tauri_fuzz_policies::filesystem::no_access_to_filenames(vec!["foo.txt".to_string()]),
        Vec::<(&str, ())>::new(),
        None,
    )
}

#[test]
#[ignore]
fn hidden_allow_read_foo_with_readonly_policy() {
    fuzz_command_with_arg(
        "read_foo_file",
        Some(mini_app::file_access::read_foo_file as usize),
        tauri_fuzz_policies::filesystem::read_only_access(),
        Vec::<(&str, ())>::new(),
        None,
    )
}

#[test]
#[ignore]
fn hidden_allow_read_foo_with_no_policy() {
    fuzz_command_with_arg(
        "read_foo_file",
        Some(mini_app::file_access::read_foo_file as usize),
        tauri_fuzz_policies::no_policy(),
        Vec::<(&str, ())>::new(),
        None,
    )
}
