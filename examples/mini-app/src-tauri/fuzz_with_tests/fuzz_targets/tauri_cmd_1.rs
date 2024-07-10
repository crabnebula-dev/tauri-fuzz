use fuzzer::tauri_utils::{create_invoke_request, invoke_command_minimal, CommandArgs};
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::webview::InvokeRequest;
mod utils;
use utils::*;

const COMMAND_NAME: &str = "tauri_cmd_1";
const COMMAND_PTR: *const () = mini_app::basic::tauri_cmd_1 as *const ();

fn setup_mock() -> tauri::WebviewWindow<MockRuntime> {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![mini_app::basic::tauri_cmd_1])
        .build(mock_context(noop_assets()))
        .expect("Failed to init Tauri app");
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    webview
}

pub fn main() {
    let w = setup_mock();
    let harness = |input: &BytesInput| {
        invoke_command_minimal(w.clone(), create_request(input.bytes()));
        ExitKind::Ok
    };
    let options =
        fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
    fuzzer::fuzz_main(
        harness,
        &options,
        COMMAND_PTR as usize,
        policies::no_policy(),
        true,
    );
}

// Helper code to create a payload tauri_cmd_1
fn create_request(bytes: &[u8]) -> InvokeRequest {
    let input = String::from_utf8_lossy(bytes).to_string();
    let arg_name = String::from("input");
    let mut args = CommandArgs::new();
    args.insert(arg_name, input);
    create_invoke_request(None, COMMAND_NAME, args)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_crash_tauri_cmd_1() {
        let options =
            fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
        let w = setup_mock();
        let harness = |_input: &BytesInput| {
            invoke_command_minimal(w.clone(), create_request("foo".as_bytes()));
            ExitKind::Ok
        };
        unsafe {
            assert!(fuzzer::fuzz_test(
                harness,
                &options,
                COMMAND_PTR as usize,
                policies::no_policy()
            )
            .is_ok());
        }
    }

    // This is a trick to capture the fuzzer exit status code.
    // The fuzzer exits the process with an error code rather than panicking.
    // This test will be started as a new process and its exit status will be captured.
    #[test]
    #[ignore]
    fn hidden_crash_tauri_cmd_1() {
        let options =
            fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
        let w = setup_mock();
        let harness = |_input: &BytesInput| {
            invoke_command_minimal(w.clone(), create_request("abc".as_bytes()));
            ExitKind::Ok
        };
        unsafe {
            let _ = fuzzer::fuzz_test(
                harness,
                &options,
                COMMAND_PTR as usize,
                policies::filesystem::no_file_access(),
            )
            .is_ok();
        }
    }

    #[test]
    fn crash_tauri_cmd_1() {
        let exe = std::env::current_exe().expect("Failed to extract current executable");
        let status = std::process::Command::new(exe)
            .args(["--ignored", "hidden_crash_tauri_cmd_1"])
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

// ::fuzzer::define_fuzz_target! {
//     command: "tauri_cmd_1",
//     path: mini_app::basic::tauri_cmd_1,
//     parameters: {
//         input: &str,
//     },
//     policy: policies::filesystem::no_file_access(),
//
