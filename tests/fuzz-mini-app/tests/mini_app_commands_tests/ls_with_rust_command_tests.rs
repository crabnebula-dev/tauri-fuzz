use crate::common::*;
// This is a trick to test fuzzers with multi-threaded and get fuzzer output when crashing.
// Frida-gum does not support multi-threads therefore we start fuzzing in different processes.
// The "hidden_*"  test will be started in a separate process and the exit status will be captured
// by the parent process/test.
#[test]
fn block_ls_with_rust_command_at_entry() {
    start_crashing_fuzz_process("hidden_block_ls_with_rust_command_output_at_entry");
    start_crashing_fuzz_process("hidden_block_ls_with_rust_command_status_at_entry");
    start_crashing_fuzz_process("hidden_block_ls_with_rust_command_spawn_at_entry")
}

#[test]
fn block_ls_with_rust_command_error_status() {
    start_crashing_fuzz_process("hidden_block_ls_with_rust_command_output_error_status");
}

#[test]
#[ignore]
fn hidden_block_ls_with_rust_command_output_at_entry() {
    fuzz_command_with_arg(
        "ls_with_rust_command_output",
        Some(mini_app::external_process::ls_with_rust_command_output as usize),
        policies::external_process::block_on_entry(vec!["ls".to_string()]),
        vec![("input", "-la")],
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_ls_with_rust_command_status_at_entry() {
    fuzz_command_with_arg(
        "ls_with_rust_command_status",
        Some(mini_app::external_process::ls_with_rust_command_status as usize),
        policies::external_process::block_on_entry(vec!["ls".to_string()]),
        vec![("input", "-la")],
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_ls_with_rust_command_spawn_at_entry() {
    fuzz_command_with_arg(
        "ls_with_rust_command_spawn",
        Some(mini_app::external_process::ls_with_rust_command_spawn as usize),
        policies::external_process::block_on_entry(vec!["ls".to_string()]),
        vec![("input", "-la")],
        None,
    )
}

#[test]
#[ignore]
fn hidden_block_ls_with_rust_command_output_error_status() {
    fuzz_command_with_arg(
        "ls_with_rust_command_output",
        Some(mini_app::external_process::ls_with_rust_command_output as usize),
        policies::external_process::block_on_error_status(),
        vec![("input", "fdsjkl")],
        None,
    )
}
