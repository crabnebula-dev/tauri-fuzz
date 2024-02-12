use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;
use tauri_fuzz_tools::{create_invoke_payload, invoke_command_minimal, CommandArgs};

const COMMAND_NAME: &str = "tauri_cmd_1";

pub fn main() {
    let ptr = mini_app::basic::tauri_cmd_1 as *const ();
    let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _ = invoke_command_minimal(app, create_payload(input.bytes()));
        ExitKind::Ok
    };

    fuzzer::main(
        harness,
        options,
        ptr as usize,
        fuzzer::policies::no_policy(),
    );
}

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder()
        .invoke_handler(tauri::generate_handler![mini_app::basic::tauri_cmd_1])
        .build(mock_context(noop_assets()))
}

// Helper code to create a payload tauri_cmd_1
fn create_payload(bytes: &[u8]) -> InvokePayload {
    let input = String::from_utf8_lossy(bytes).to_string();
    let arg_name = String::from("input");
    let mut args = CommandArgs::new();
    args.insert(arg_name, input);
    create_invoke_payload(None, COMMAND_NAME, args)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_crash_tauri_cmd_1() {
        let addr = mini_app::basic::tauri_cmd_1 as *const ();
        let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
        let harness = |_input: &BytesInput| {
            let app = setup_tauri_mock().expect("Failed to init Tauri app");
            let _res = invoke_command_minimal(app, create_payload("foo".as_bytes()));
            ExitKind::Ok
        };
        unsafe {
            assert!(fuzzer::fuzz_test(
                harness,
                &options,
                addr as usize,
                fuzzer::policies::no_policy()
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
        let addr = mini_app::basic::tauri_cmd_1 as *const ();
        let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
        let harness = |_input: &BytesInput| {
            let app = setup_tauri_mock().expect("Failed to init Tauri app");
            let _res = invoke_command_minimal(app, create_payload("abc".as_bytes()));
            ExitKind::Ok
        };
        unsafe {
            let _ = fuzzer::fuzz_test(
                harness,
                &options,
                addr as usize,
                fuzzer::policies::file_policy::no_file_access(),
            )
            .is_ok();
        }
    }

    #[test]
    fn crash_tauri_cmd_1() {
        let exe = std::env::current_exe().expect("Failed to extract current executable");
        let status = std::process::Command::new(exe)
            .args(&["--ignored", "hidden_crash_tauri_cmd_1"])
            .status()
            .expect("Unable to run program");

        assert_eq!(Some(134), status.code());
    }
}
