// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use fuzz_mini_app::utils::fuzz_command_with_arg;
use tauri_fuzz::tauri::{start_crashing_fuzz_process, start_non_crashing_fuzz_process};

// This is a trick to test fuzzers with multi-threaded and get fuzzer output when crashing.
// Frida-gum does not support multi-threads therefore we start fuzzing in different processes.
// The "hidden_*"  test will be started in a separate process and the exit status will be captured
// by the parent process/test.

#[cfg(not(target_os = "windows"))]
#[test]
fn allow_ls_with_rust_with_no_policy() {
    start_non_crashing_fuzz_process("hidden_allow_ls_with_rust_command_output_no_policy");
    start_non_crashing_fuzz_process("hidden_allow_ls_with_rust_command_status_no_policy");
    start_non_crashing_fuzz_process("hidden_allow_ls_with_rust_command_spawn_no_policy");
}

#[cfg(not(target_os = "windows"))]
#[test]
fn block_ls_with_rust_api_at_entry() {
    start_crashing_fuzz_process("hidden_block_ls_with_rust_command_output_at_entry");
    start_crashing_fuzz_process("hidden_block_ls_with_rust_command_status_at_entry");
    start_crashing_fuzz_process("hidden_block_ls_with_rust_command_spawn_at_entry");
}

#[cfg(not(target_os = "windows"))]
#[test]
fn block_ls_with_rust_api_error_status() {
    start_crashing_fuzz_process("hidden_block_ls_with_rust_command_output_error_status");
    start_crashing_fuzz_process("hidden_block_ls_with_rust_command_status_error_status");
    start_crashing_fuzz_process("hidden_block_ls_with_rust_command_wait_error_status");
}

#[test]
fn block_ls_with_error_status() {
    start_crashing_fuzz_process("hidden_block_ls_with_error_status_from_rust_command_output");
    start_crashing_fuzz_process("hidden_block_ls_with_error_status_from_rust_command_status");
    start_crashing_fuzz_process("hidden_block_ls_with_error_status_from_rust_command_spawn");
}

#[test]
fn allow_ls_with_ok_status() {
    start_non_crashing_fuzz_process("hidden_allow_ls_with_ok_status_from_rust_command_output");
    start_non_crashing_fuzz_process("hidden_allow_ls_with_ok_status_from_rust_command_status");
    start_non_crashing_fuzz_process("hidden_allow_ls_with_ok_status_from_rust_command_spawn");
}

#[cfg(not(target_os = "windows"))]
const BLOCKED_BINARY: &str = "ls";
#[cfg(target_os = "windows")]
const BLOCKED_BINARY: &str = "cmd";

#[cfg(not(target_os = "windows"))]
const CORRECT_ARG: &str = "-la";
#[cfg(target_os = "windows")]
const CORRECT_ARG: &str = "";
const WRONG_ARG: &str = "foo";

#[test]
#[ignore]
fn hidden_block_ls_with_rust_command_output_at_entry() {
    fuzz_command_with_arg(
        "ls_with_rust_command_output",
        Some(mini_app::external_process::ls_with_rust_command_output as usize),
        tauri_fuzz_policies::external_process::block_on_entry(vec![BLOCKED_BINARY.to_string()]),
        vec![("input", CORRECT_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_ls_with_rust_command_status_at_entry() {
    fuzz_command_with_arg(
        "ls_with_rust_command_status",
        Some(mini_app::external_process::ls_with_rust_command_status as usize),
        tauri_fuzz_policies::external_process::block_on_entry(vec![BLOCKED_BINARY.to_string()]),
        vec![("input", CORRECT_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_ls_with_rust_command_spawn_at_entry() {
    fuzz_command_with_arg(
        "ls_with_rust_command_spawn",
        Some(mini_app::external_process::ls_with_rust_command_spawn as usize),
        tauri_fuzz_policies::external_process::block_on_entry(vec![BLOCKED_BINARY.to_string()]),
        vec![("input", CORRECT_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_ls_with_rust_command_output_error_status() {
    fuzz_command_with_arg(
        "ls_with_rust_command_output",
        Some(mini_app::external_process::ls_with_rust_command_output as usize),
        tauri_fuzz_policies::external_process::block_on_rust_api_error_status(),
        vec![("input", WRONG_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_ls_with_rust_command_status_error_status() {
    fuzz_command_with_arg(
        "ls_with_rust_command_status",
        Some(mini_app::external_process::ls_with_rust_command_status as usize),
        tauri_fuzz_policies::external_process::block_on_rust_api_error_status(),
        vec![("input", WRONG_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_ls_with_rust_command_wait_error_status() {
    fuzz_command_with_arg(
        "ls_with_rust_command_spawn",
        Some(mini_app::external_process::ls_with_rust_command_spawn as usize),
        tauri_fuzz_policies::external_process::block_on_rust_api_error_status(),
        vec![("input", WRONG_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_ls_with_error_status_from_rust_command_output() {
    fuzz_command_with_arg(
        "ls_with_rust_command_output",
        Some(mini_app::external_process::ls_with_rust_command_output as usize),
        tauri_fuzz_policies::external_process::block_on_child_process_error_status(),
        vec![("input", WRONG_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_ls_with_error_status_from_rust_command_status() {
    fuzz_command_with_arg(
        "ls_with_rust_command_status",
        Some(mini_app::external_process::ls_with_rust_command_status as usize),
        tauri_fuzz_policies::external_process::block_on_child_process_error_status(),
        vec![("input", WRONG_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_ls_with_error_status_from_rust_command_spawn() {
    fuzz_command_with_arg(
        "ls_with_rust_command_spawn",
        Some(mini_app::external_process::ls_with_rust_command_spawn as usize),
        tauri_fuzz_policies::external_process::block_on_child_process_error_status(),
        vec![("input", WRONG_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_allow_ls_with_ok_status_from_rust_command_output() {
    fuzz_command_with_arg(
        "ls_with_rust_command_output",
        Some(mini_app::external_process::ls_with_rust_command_output as usize),
        tauri_fuzz_policies::external_process::block_on_child_process_error_status(),
        vec![("input", CORRECT_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_allow_ls_with_ok_status_from_rust_command_status() {
    fuzz_command_with_arg(
        "ls_with_rust_command_status",
        Some(mini_app::external_process::ls_with_rust_command_status as usize),
        tauri_fuzz_policies::external_process::block_on_child_process_error_status(),
        vec![("input", CORRECT_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_allow_ls_with_ok_status_from_rust_command_spawn() {
    fuzz_command_with_arg(
        "ls_with_rust_command_spawn",
        Some(mini_app::external_process::ls_with_rust_command_spawn as usize),
        tauri_fuzz_policies::external_process::block_on_child_process_error_status(),
        vec![("input", CORRECT_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_allow_ls_with_rust_command_output_no_policy() {
    fuzz_command_with_arg(
        "ls_with_rust_command_output",
        Some(mini_app::external_process::ls_with_rust_command_output as usize),
        tauri_fuzz_policies::no_policy(),
        vec![("input", CORRECT_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_allow_ls_with_rust_command_status_no_policy() {
    fuzz_command_with_arg(
        "ls_with_rust_command_status",
        Some(mini_app::external_process::ls_with_rust_command_status as usize),
        tauri_fuzz_policies::no_policy(),
        vec![("input", CORRECT_ARG)],
        None,
    )
}

#[test]
#[ignore]
fn hidden_allow_ls_with_rust_command_spawn_no_policy() {
    fuzz_command_with_arg(
        "ls_with_rust_command_spawn",
        Some(mini_app::external_process::ls_with_rust_command_spawn as usize),
        tauri_fuzz_policies::no_policy(),
        vec![("input", CORRECT_ARG)],
        None,
    )
}
