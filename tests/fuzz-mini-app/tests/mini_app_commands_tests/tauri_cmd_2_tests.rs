// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use appfuzz_rt::tauri::{start_crashing_fuzz_process, start_non_crashing_fuzz_process};
use fuzz_mini_app::utils::fuzz_command_with_arg;

// This is a trick to test fuzzers with multi-threaded and get fuzzer output when crashing.
// Frida-gum does not support multi-threads therefore we start fuzzing in different processes.
// The "hidden_*"  test will be started in a separate process and the exit status will be captured
// by the parent process/test.
#[test]
fn crash_tauri_cmd_2() {
    start_crashing_fuzz_process("hidden_crash_tauri_cmd_2")
}
#[test]
#[ignore]
fn hidden_crash_tauri_cmd_2() {
    fuzz_command_with_arg(
        "tauri_cmd_2",
        Some(mini_app::basic::tauri_cmd_2 as usize),
        policies::no_policy(),
        vec![("input", 100u32)],
        None,
    )
}

#[test]
fn no_crash_tauri_cmd_2() {
    start_non_crashing_fuzz_process("hidden_no_crash_tauri_cmd_2")
}
#[test]
#[ignore]
fn hidden_no_crash_tauri_cmd_2() {
    fuzz_command_with_arg(
        "tauri_cmd_2",
        Some(mini_app::basic::tauri_cmd_2 as usize),
        policies::no_policy(),
        vec![("input", 1u32)],
        None,
    )
}
