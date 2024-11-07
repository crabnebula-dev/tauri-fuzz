// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use fuzz_mini_app::utils::fuzz_command_with_arg;
use tauri_fuzz::tauri::start_crashing_fuzz_process;
// This is a trick to test fuzzers with multi-threaded and get fuzzer output when crashing.
// Frida-gum does not support multi-threads therefore we start fuzzing in different processes.
// The "hidden_*"  test will be started in a separate process and the exit status will be captured
// by the parent process/test.
#[test]
fn direct_panic() {
    start_crashing_fuzz_process("hidden_direct_panic")
}
#[test]
#[ignore]
fn hidden_direct_panic() {
    fuzz_command_with_arg(
        "direct_panic",
        Some(mini_app::basic::direct_panic as usize),
        tauri_fuzz_policies::no_policy(),
        Vec::<(&str, ())>::new(),
        None,
    )
}
