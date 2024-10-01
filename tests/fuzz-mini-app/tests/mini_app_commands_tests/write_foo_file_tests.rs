// // Copyright 2024-2022 CrabNebula Ltd.
// // SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use fuzz_mini_app::utils::fuzz_command_with_arg;
use fuzzer::tauri::{start_crashing_fuzz_process, start_non_crashing_fuzz_process};

// This is a trick to test fuzzers with multi-threaded and get fuzzer output when crashing.
// Frida-gum does not support multi-threads therefore we start fuzzing in different processes.
// The "hidden_*"  test will be started in a separate process and the exit status will be captured
// by the parent process/test.
#[test]
fn block_all_file_access() {
    start_crashing_fuzz_process("hidden_block_all_file_access")
}

// Block writing foo with no access to files with name "foo.txt"
#[test]
fn block_by_filename() {
    start_crashing_fuzz_process("hidden_block_by_filename")
}

// Block writing to foo.txt due to read only policy
#[test]
fn block_by_readonly_policy() {
    start_crashing_fuzz_process("hidden_block_by_readonly_policy")
}

// writeonly policy and no policy should not crash
#[test]
fn allowed_by_writeonly_policy() {
    start_non_crashing_fuzz_process("hidden_allow_by_writeonly_policy");
    start_non_crashing_fuzz_process("hidden_allow_by_no_policy");
}

#[test]
#[ignore]
fn hidden_block_all_file_access() {
    fuzz_command_with_arg(
        "write_foo_file",
        Some(mini_app::file_access::write_foo_file as usize),
        policies::filesystem::no_file_access(),
        vec![("input", "foo")],
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_by_filename() {
    fuzz_command_with_arg(
        "write_foo_file",
        Some(mini_app::file_access::write_foo_file as usize),
        policies::filesystem::no_access_to_filenames(vec!["foo.txt".to_string()]),
        vec![("input", "foo")],
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_by_readonly_policy() {
    fuzz_command_with_arg(
        "write_foo_file",
        Some(mini_app::file_access::write_foo_file as usize),
        policies::filesystem::read_only_access(),
        vec![("input", "foo")],
        None,
    )
}

#[test]
#[ignore]
fn hidden_allow_by_writeonly_policy() {
    fuzz_command_with_arg(
        "write_foo_file",
        Some(mini_app::file_access::write_foo_file as usize),
        policies::filesystem::write_only_access(),
        vec![("input", "foo")],
        None,
    )
}

#[test]
#[ignore]
fn hidden_allow_by_no_policy() {
    fuzz_command_with_arg(
        "write_foo_file",
        Some(mini_app::file_access::write_foo_file as usize),
        policies::no_policy(),
        vec![("input", "foo")],
        None,
    )
}
