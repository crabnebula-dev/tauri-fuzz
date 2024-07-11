#![allow(non_snake_case)]
use fuzz_mini_app::utils::path_to_foo;

use crate::common::*;
// This is a trick to test fuzzers with multi-threaded and get fuzzer output when crashing.
// Frida-gum does not support multi-threads therefore we start fuzzing in different processes.
// The "hidden_*"  test will be started in a separate process and the exit status will be captured
// by the parent process/test.
#[test]
#[ignore]
fn fs_readFile_no_policy() {
    start_non_crashing_fuzz_process("hidden_fs_readFile_no_policy")
}
#[test]
#[ignore]
fn hidden_fs_readFile_no_policy() {
    fuzz_command_with_arg(
        "fs_readFile",
        None,
        policies::no_policy(),
        vec![("path", path_to_foo())],
        Some("Fs".into()),
    )
}

#[test]
#[ignore]
fn fs_readFile_block_files() {
    start_crashing_fuzz_process("hidden_fs_readFile_block_files")
}
#[test]
#[ignore]
fn hidden_fs_readFile_block_files() {
    fuzz_command_with_arg(
        "fs_readFile",
        None,
        policies::no_policy(),
        vec![("path", path_to_foo())],
        Some("Fs".into()),
    )
}
