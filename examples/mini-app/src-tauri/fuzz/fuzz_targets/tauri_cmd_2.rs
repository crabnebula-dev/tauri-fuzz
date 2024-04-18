use fuzzer::tauri_utils::{create_invoke_payload, invoke_command_minimal, CommandArgs};
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;
mod utils;
use utils::*;

const COMMAND_NAME: &str = "tauri_cmd_2";

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder()
        .invoke_handler(tauri::generate_handler![mini_app::basic::tauri_cmd_2])
        .build(mock_context(noop_assets()))
}

pub fn main() {
    let addr = mini_app::basic::tauri_cmd_2 as *const ();
    let options =
        fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        invoke_command_minimal(app, create_payload(input.bytes()));
        ExitKind::Ok
    };
    fuzzer::fuzz_main(harness, options, addr as usize, policies::no_policy());
}

// Helper code to create a payload tauri_cmd_2
fn create_payload(bytes: &[u8]) -> InvokePayload {
    let input = bytes_input_to_u32(bytes);
    let arg_name = String::from("input");
    let mut args = CommandArgs::new();
    args.insert(arg_name, input);
    create_invoke_payload(None, COMMAND_NAME, args)
}

fn bytes_input_to_u32(bytes_input: &[u8]) -> u32 {
    let mut array_input = [0u8; 4];
    for (dst, src) in array_input.iter_mut().zip(bytes_input) {
        *dst = *src
    }
    u32::from_be_bytes(array_input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_crash_tauri_cmd_2() {
        let addr = mini_app::basic::tauri_cmd_2 as *const ();
        let options =
            fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
        let harness = |_input: &BytesInput| {
            let app = setup_tauri_mock().expect("Failed to init Tauri app");
            invoke_command_minimal(app, create_payload(&1u32.to_be_bytes()));
            ExitKind::Ok
        };
        unsafe {
            assert!(
                fuzzer::fuzz_test(harness, &options, addr as usize, policies::no_policy()).is_ok()
            );
        }
    }

    // This is a trick to capture the fuzzer exit status code.
    // The fuzzer exits the process with an error code rather than panicking.
    // This test will be started as a new process and its exit status will be captured.
    #[test]
    #[ignore]
    fn hidden_crash_tauri_cmd_2() {
        let addr = mini_app::basic::tauri_cmd_2 as *const ();
        let options =
            fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
        let harness = |_input: &BytesInput| {
            let app = setup_tauri_mock().expect("Failed to init Tauri app");
            invoke_command_minimal(app, create_payload(&100u32.to_be_bytes()));
            ExitKind::Ok
        };
        unsafe {
            let _ = fuzzer::fuzz_test(
                harness,
                &options,
                addr as usize,
                policies::file_policy::no_file_access(),
            )
            .is_ok();
        }
    }

    #[test]
    fn crash_tauri_cmd_2() {
        let exe = std::env::current_exe().expect("Failed to extract current executable");
        let status = std::process::Command::new(exe)
            .args(["--ignored", "hidden_crash_tauri_cmd_2"])
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
