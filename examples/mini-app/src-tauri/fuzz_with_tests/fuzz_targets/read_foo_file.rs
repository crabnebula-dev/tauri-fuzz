use fuzzer::tauri_utils::{create_invoke_request, invoke_command_minimal, CommandArgs};
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::webview::InvokeRequest;
mod utils;
use utils::*;

const COMMAND_NAME: &str = "read_foo_file";
const COMMAND_PTR: *const () = mini_app::file_access::read_foo_file as *const ();

fn setup_mock() -> tauri::WebviewWindow<MockRuntime> {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            mini_app::file_access::read_foo_file
        ])
        .build(mock_context(noop_assets()))
        .expect("Failed to init Tauri app");
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    webview
}

pub fn main() {
    let options = fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir());
    let w = setup_mock();
    let harness = |input: &BytesInput| {
        invoke_command_minimal(w.clone(), create_request(input.bytes()));
        ExitKind::Ok
    };

    fuzzer::fuzz_main(
        harness,
        options.into(),
        COMMAND_PTR as usize,
        policies::filesystem::no_access_to_filenames(),
    );
}

fn create_request(_bytes: &[u8]) -> InvokeRequest {
    let args = CommandArgs::new();
    create_invoke_request(None, COMMAND_NAME, args)
}

#[cfg(test)]
mod test {
    use super::*;

    // This is a trick to capture the fuzzer exit status code.
    // The fuzzer exits the process with an error code rather than panicking.
    // This test will be started as a new process and its exit status will be captured.
    #[test]
    #[ignore]
    fn hidden_read_foo_block_all_file_access() {
        let options =
            fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir());
        let w = setup_mock();
        let harness = |input: &BytesInput| {
            invoke_command_minimal(w.clone(), create_request(input.bytes()));
            ExitKind::Ok
        };
        unsafe {
            let _ = fuzzer::fuzz_test(
                harness,
                &options.into(),
                COMMAND_PTR as usize,
                policies::filesystem::no_file_access(),
            )
            .is_ok();
        }
    }

    // This is a trick to capture the fuzzer exit status code.
    // The fuzzer exits the process with an error code rather than panicking.
    // This test will be started as a new process and its exit status will be captured.
    #[test]
    #[ignore]
    fn hidden_read_foo_block_access_to_foo() {
        let w = setup_mock();
        let options =
            fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir());
        let harness = |input: &BytesInput| {
            invoke_command_minimal(w.clone(), create_request(input.bytes()));
            ExitKind::Ok
        };
        unsafe {
            let _ = fuzzer::fuzz_test(
                harness,
                &options.into(),
                COMMAND_PTR as usize,
                policies::filesystem::no_access_to_filenames(),
            )
            .is_ok();
        }
    }

    // Block reading foo with no file access policy
    #[test]
    fn read_foo_block_all_file_access() {
        let exe = std::env::current_exe().expect("Failed to extract current executable");
        let status = std::process::Command::new(exe)
            .args(["--ignored", "hidden_read_foo_block_all_file_access"])
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

    // Block reading foo with no access to files with name "foo.txt"
    #[test]
    fn read_foo_block_access_by_filename() {
        let exe = std::env::current_exe().expect("Failed to extract current executable");
        let status = std::process::Command::new(exe)
            .args(["--ignored", "hidden_read_foo_block_access_to_foo"])
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

    // Read-only access should not block `read_foo`
    #[test]
    fn read_foo_accepted_by_readonly_policy() {
        let w = setup_mock();
        let options =
            fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir());
        let harness = |input: &BytesInput| {
            invoke_command_minimal(w.clone(), create_request(input.bytes()));
            ExitKind::Ok
        };
        unsafe {
            assert!(fuzzer::fuzz_test(
                harness,
                &options.into(),
                COMMAND_PTR as usize,
                policies::filesystem::read_only_access(),
            )
            .is_ok(),)
        }
    }
}