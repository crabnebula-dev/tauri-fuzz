use fuzz_mini_app::utils::*;
use fuzzer::tauri_utils::invoke_command_minimal;
use fuzzer::tauri_utils::{create_invoke_request, CommandArgs};
use libafl::inputs::BytesInput;
use libafl::prelude::ExitKind;
use policies::engine::FuzzPolicy;

// This is a trick to detect a crash while fuzzing.
// The fuzzer exits the process with an error code rather than panicking.
// This test will be started as a new process and its exit status will be captured.
pub(crate) fn start_crashing_fuzz_process(test_name: &str) {
    let exe = std::env::current_exe().expect("Failed to extract current executable");
    let status = std::process::Command::new(exe)
        .args(["--ignored", test_name])
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

// This is a trick to detect a fuzzing output.
// We start the fuzzer in a new process because Frida does not support multi-threaded
// Starting multiple tests in multiple threads will cause the tests to hang
pub(crate) fn start_non_crashing_fuzz_process(test_name: &str) {
    let exe = std::env::current_exe().expect("Failed to extract current executable");
    let status = std::process::Command::new(exe)
        .args(["--ignored", test_name])
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

pub(crate) fn fuzz_command_with_arg<T>(
    command_name: &str,
    command_ptr: Option<usize>,
    policy: FuzzPolicy,
    args: Vec<(&str, T)>,
    tauri_plugin: Option<String>,
) where
    T: serde::ser::Serialize + Clone,
{
    let options =
        fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), command_name, fuzz_dir()).into();
    let w = setup_mock();
    let harness = |_input: &BytesInput| {
        let mut command_args = CommandArgs::new();
        for arg in args.clone().into_iter() {
            command_args.insert(arg.0, arg.1.clone());
        }
        let request = create_invoke_request(tauri_plugin.clone(), command_name, command_args);
        invoke_command_minimal(w.clone(), request);
        ExitKind::Ok
    };

    let monitored_code = command_ptr.unwrap_or_else(|| std::ptr::addr_of!(harness) as usize);

    fuzzer::fuzz_main(
        harness,
        &options,
        monitored_code,
        // command_ptr,
        policy,
        true,
    )
}
