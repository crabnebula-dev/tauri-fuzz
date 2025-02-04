// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use fuzz_mini_app::utils::fuzz_command_with_arg;
use tauri_fuzz::tauri::{start_crashing_fuzz_process, start_non_crashing_fuzz_process};

// This is a trick to test fuzzers with multi-threaded and get fuzzer output when crashing.
// Frida-gum does not support multi-threads therefore we start fuzzing in different processes.
// The "hidden_*"  test will be started in a separate process and the exit status will be captured
// by the parent process/test.
#[test]
fn crash_fopen() {
    start_crashing_fuzz_process("hidden_crash_fopen")
}

#[test]
fn no_crash_fopen() {
    start_non_crashing_fuzz_process("hidden_no_crash_fopen")
}

#[test]
#[ignore]
fn hidden_crash_fopen() {
    fuzz_command_with_arg(
        "fopen",
        Some(mini_app::libc_calls::fopen as usize),
        tauri_fuzz_policies::filesystem::no_file_access(),
        vec![("filename", "/tmp/foo"), ("mode", "w")],
        None,
    )
}
#[test]
#[ignore]
fn hidden_no_crash_fopen() {
    fuzz_command_with_arg(
        "fopen",
        Some(mini_app::libc_calls::fopen as usize),
        tauri_fuzz_policies::no_policy(),
        vec![("filename", "/tmp/foo"), ("mode", "w")],
        None,
    )
}
