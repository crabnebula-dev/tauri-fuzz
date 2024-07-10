use fuzzer::tauri_utils::{create_invoke_request, invoke_command_minimal, CommandArgs};
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::webview::InvokeRequest;
mod utils;
use utils::*;

const COMMAND_NAME: &str = "direct_panic";
const COMMAND_PTR: *const () = mini_app::basic::direct_panic as *const ();

fn setup_mock() -> tauri::WebviewWindow<MockRuntime> {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![mini_app::basic::direct_panic])
        .build(mock_context(noop_assets()))
        .expect("Failed to init Tauri app");
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    webview
}

pub fn main() {
    let options =
        fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
    let w = setup_mock();
    let harness = |input: &BytesInput| {
        invoke_command_minimal(w.clone(), create_request(input.bytes()));
        ExitKind::Ok
    };
    fuzzer::fuzz_main(
        harness,
        &options,
        COMMAND_PTR as usize,
        policies::no_policy(),
        false,
    );
}

#[allow(unused_variables)]
fn create_request(bytes: &[u8]) -> InvokeRequest {
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
    fn real_test_direct_panic() {
        let options =
            fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
        let w = setup_mock();
        let harness = |input: &BytesInput| {
            invoke_command_minimal(w.clone(), create_request(input.bytes()));
            ExitKind::Ok
        };
        unsafe {
            let _ = fuzzer::fuzz_test(
                harness,
                &options,
                COMMAND_PTR as usize,
                policies::no_policy(),
            )
            .is_ok();
        }
    }

    // Start another process that will actually launch the fuzzer
    #[test]
    fn test_direct_panic() {
        let exe = std::env::current_exe().expect("Failed to extract current executable");
        let status = std::process::Command::new(exe)
            .args(["--ignored", "real_test_direct_panic"])
            .status()
            .expect("Unable to run program");

        if cfg!(target_os = "windows") {
            assert_eq!(Some(1), status.code());
        } else {
            assert_eq!(Some(134), status.code());
        }
    }
}
