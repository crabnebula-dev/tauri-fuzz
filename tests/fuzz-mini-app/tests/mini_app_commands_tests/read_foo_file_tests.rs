use crate::common::*;
// This is a trick to test fuzzers with multi-threaded and get fuzzer output when crashing.
// Frida-gum does not support multi-threads therefore we start fuzzing in different processes.
// The "hidden_*"  test will be started in a separate process and the exit status will be captured
// by the parent process/test.
#[test]
fn block_all_file_access() {
    start_crashing_fuzz_process("hidden_block_all_file_access")
}
#[test]
#[ignore]
fn hidden_block_all_file_access() {
    fuzz_command_with_arg(
        "read_foo_file",
        Some(mini_app::file_access::read_foo_file as usize),
        policies::filesystem::no_file_access(),
        Vec::<(&str, ())>::new(),
        None,
    )
}

// Block reading foo with no access to files with name "foo.txt"
#[test]
fn block_by_filename() {
    start_crashing_fuzz_process("hidden_block_by_filename")
}
#[test]
#[ignore]
fn hidden_block_by_filename() {
    fuzz_command_with_arg(
        "read_foo_file",
        Some(mini_app::file_access::read_foo_file as usize),
        policies::filesystem::no_access_to_filenames(vec!["foo.txt".to_string()]),
        Vec::<(&str, ())>::new(),
        None,
    )
}

// Read-only access should not block `read_foo`
#[test]
fn allow_by_readonly_policy() {
    start_non_crashing_fuzz_process("hidden_allow_by_readonly_policy")
}
#[test]
#[ignore]
fn hidden_allow_by_readonly_policy() {
    fuzz_command_with_arg(
        "read_foo_file",
        Some(mini_app::file_access::read_foo_file as usize),
        policies::filesystem::read_only_access(),
        Vec::<(&str, ())>::new(),
        None,
    )
}
