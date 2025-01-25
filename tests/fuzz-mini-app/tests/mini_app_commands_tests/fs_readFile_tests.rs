// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(non_snake_case)]
// use crate::mini_app_commands_tests::path_to_foo;
use fuzz_mini_app::utils::fuzz_command_with_arg;
use fuzz_mini_app::utils::path_to_foo;
use tauri_fuzz::tauri::{start_crashing_fuzz_process, start_non_crashing_fuzz_process};

// This is a trick to test fuzzers with multi-threaded and get fuzzer output when crashing.
// Frida-gum does not support multi-threads therefore we start fuzzing in different processes.
// The "hidden_*"  test will be started in a separate process and the exit status will be captured
// by the parent process/test.

/// TODO: resolve these tests
/// It's currently disabled because fs_readFile is now an async command and
/// the fuzzer currently does not handle it
#[ignore]
#[test]
fn allow_fs_readFile() {
    start_non_crashing_fuzz_process("hidden_fs_readFile_no_policy");
    start_non_crashing_fuzz_process("hidden_fs_readFile_readonly_policy");
}

/// TODO: resolve these tests
/// It's currently disabled because fs_readFile is now an async command and
/// the fuzzer currently does not handle it
#[ignore]
#[test]
fn block_fs_readFile() {
    start_crashing_fuzz_process("hidden_fs_readFile_block_files");
    start_crashing_fuzz_process("hidden_fs_readFile_writeonly_policy");
}

#[test]
#[ignore]
fn hidden_fs_readFile_no_policy() {
    fuzz_command_with_arg(
        "read_file",
        None,
        tauri_fuzz_policies::no_policy(),
        vec![("path", path_to_foo())],
        Some("fs".into()),
    )
}

#[test]
#[ignore]
fn hidden_fs_readFile_block_files() {
    fuzz_command_with_arg(
        "read_file",
        None,
        tauri_fuzz_policies::filesystem::no_file_access(),
        vec![("path", path_to_foo())],
        Some("fs".into()),
    )
}

#[test]
#[ignore]
fn hidden_fs_readFile_writeonly_policy() {
    fuzz_command_with_arg(
        "read_file",
        None,
        tauri_fuzz_policies::filesystem::write_only_access(),
        vec![("path", path_to_foo())],
        Some("fs".into()),
    )
}

#[test]
#[ignore]
fn hidden_fs_readFile_readeonly_policy() {
    fuzz_command_with_arg(
        "read_file",
        None,
        tauri_fuzz_policies::filesystem::read_only_access(),
        vec![("path", path_to_foo())],
        Some("fs".into()),
    )
}
