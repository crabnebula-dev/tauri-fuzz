#![allow(non_snake_case)]
use fuzzer::tauri_utils::{create_invoke_payload, invoke_command_minimal, CommandArgs};
/// We fuzz the Tauri command `readFile` in the `Fs` module
/// The calling convention for the `InvokePayload` are different from custom Tauri commands
use libafl::inputs::BytesInput;
use libafl::prelude::ExitKind;
use std::path::PathBuf;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;
use tauri_utils::config::FsAllowlistScope;
mod utils;
use utils::*;

const COMMAND_NAME: &str = "readFile";

/// Generate a Tauri mock runtime
fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    let mut context = mock_context(noop_assets());
    // Allow the module to read in "../assets/" directory
    context.config_mut().tauri.allowlist.fs.scope =
        FsAllowlistScope::AllowedPaths(vec![path_to_foo()]);

    // We're not using the usual `mock_builder_minimal` since the function we're fuzzing uses a
    // state
    mock_builder()
        .invoke_handler(tauri::generate_handler![])
        .build(context)
}

fn path_to_foo() -> PathBuf {
    let mut assets_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    assets_dir.pop();
    assets_dir.push("assets");
    assets_dir.push("foo.txt");
    assets_dir
}

/// The harness that we are going to fuzz
fn harness(_input: &BytesInput) -> ExitKind {
    let app = setup_tauri_mock().expect("Failed to init Tauri app");
    let foo_path = path_to_foo().to_string_lossy().into_owned();
    invoke_command_minimal(app, create_payload(foo_path.as_bytes()));
    ExitKind::Ok
}

pub fn main() {
    // The function in which the fuzzer analysis will be applied
    let fuzzed_function = crate::harness as *const ();
    let options =
        fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
    fuzzer::fuzz_main(
        harness,
        options,
        fuzzed_function as usize,
        policies::file_policy::no_file_access(),
    );
}

/// Create an `InvokePayload` for a Tauri module command
fn create_payload(bytes: &[u8]) -> InvokePayload {
    let mut args = CommandArgs::new();
    args.insert("path", String::from_utf8_lossy(bytes).into_owned());
    create_invoke_payload(Some("Fs".into()), COMMAND_NAME, args)
}

#[cfg(test)]
mod test {
    use super::*;

    // This is a trick to capture the fuzzer exit status code when finding a crash.
    // The fuzzer exits the process with an error code rather than panicking.
    // This test will be started as a new process and its exit status will be captured.
    #[test]
    #[ignore]
    fn real_test_fs_readFile() {
        let addr = crate::harness as *const ();
        let options =
            fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
        unsafe {
            let _ = fuzzer::fuzz_test(
                crate::harness,
                &options,
                addr as usize,
                policies::file_policy::no_file_access(),
            )
            .is_ok();
        }
    }

    /// Start another process that will actually launch the fuzzer
    #[test]
    fn test_fs_readFile() {
        let exe = std::env::current_exe().expect("Failed to extract current executable");
        let status = std::process::Command::new(exe)
            .args(["--ignored", "real_test_fs_readFile"])
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
}
