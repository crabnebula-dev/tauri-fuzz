#![allow(non_snake_case)]
use fuzzer::tauri_utils::{
    create_invoke_request, invoke_command_minimal, setup_context_with_plugin, CommandArgs,
};
/// We fuzz the Tauri command `readFile` in the `Fs` module
/// The calling convention for the `InvokeRequest` are different from custom Tauri commands
use libafl::inputs::BytesInput;
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::webview::InvokeRequest;
mod utils;
use tauri_plugin_fs::FsExt;
use utils::*;

const COMMAND_NAME: &str = "readFile";

fn setup_mock() -> tauri::WebviewWindow<MockRuntime> {
    const FS_READ_FILE_PERMISSION: &str = r#"
[[permission]]
identifier = "read-files"
description = "This enables file read related commands without any pre-configured accessible paths."
commands.allow = [
    "read_file",
]"#;

    // Capability given to our mock app, use `fs:read-files` permission
    const CAPABILITY: &str = r#"{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "fs:read-files"
  ]
}"#;

    // Prepare context with right permissions and capability
    let mut context = mock_context(noop_assets());
    setup_context_with_plugin(&mut context, "fs", FS_READ_FILE_PERMISSION, CAPABILITY);

    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            mini_app::file_access::read_foo_file
        ])
        .build(context)
        .expect("Failed to init Tauri app");

    // Modify the scope of the fs plugin
    let scope = app.fs_scope();
    scope.allow_file(path_to_foo().to_str().unwrap());

    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    webview
}

/// The harness that we are going to fuzz
fn harness(_input: &BytesInput) -> ExitKind {
    let w = setup_mock();
    let foo_path = path_to_foo().to_string_lossy().into_owned();
    invoke_command_minimal(w, create_request(foo_path.as_bytes()));
    ExitKind::Ok
}

pub fn main() {
    // The function in which the fuzzer analysis will be applied
    let fuzzed_function = crate::harness as *const ();
    let options =
        fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
    fuzzer::fuzz_main(
        harness,
        &options,
        fuzzed_function as usize,
        policies::filesystem::no_file_access(),
        false,
    );
}

/// Create an `InvokeRequest` for a Tauri module command
fn create_request(bytes: &[u8]) -> InvokeRequest {
    let mut args = CommandArgs::new();
    args.insert("path", String::from_utf8_lossy(bytes).into_owned());
    create_invoke_request(Some("Fs".into()), COMMAND_NAME, args)
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
                policies::filesystem::no_file_access(),
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
