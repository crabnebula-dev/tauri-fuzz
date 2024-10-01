// // Copyright 2024-2022 CrabNebula Ltd., Alexandre Dang
// // SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

// This is a trick to detect a crash while fuzzing when doing tests.
// The fuzzer exits the process with an error code rather than panicking.
// This test will be started as a new process and its exit status will be captured.
pub fn start_crashing_fuzz_process(test_name: &str) {
    let exe = std::env::current_exe().expect("Failed to extract current executable");
    let status = std::process::Command::new(exe)
        .args(["--ignored", test_name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("Unable to run program");

    if cfg!(target_os = "windows") {
        // Check that fuzzer process launched exit with status error 1
        assert_eq!(Some(1), status.code());
    } else {
        // Check that fuzzer process launched exit with status error 134
        assert_eq!(Some(134), status.code());
    }
}

// This is a trick to detect a fuzzing output when doing tests.
// We start the fuzzer in a new process because Frida does not support multi-threaded
// Starting multiple tests in multiple threads will cause the tests to hang
pub fn start_non_crashing_fuzz_process(test_name: &str) {
    let exe = std::env::current_exe().expect("Failed to extract current executable");
    let status = std::process::Command::new(exe)
        .args(["--ignored", test_name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("Unable to run program");

    if cfg!(target_os = "windows") {
        // Check that fuzzer process launched exit with status error 1
        assert_eq!(Some(0), status.code());
    } else {
        // Check that fuzzer process launched exit with status error 134
        assert_eq!(Some(0), status.code());
    }
}
