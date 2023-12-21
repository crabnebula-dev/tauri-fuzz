use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;
use tauri_fuzz_tools::{
    create_invoke_payload, fuzzer, get_options, invoke_command_minimal, mock_builder_minimal,
    CommandArgs,
};

const COMMAND_NAME: &str = "fopen";

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder_minimal()
        .invoke_handler(tauri::generate_handler![mini_app::libc_calls::fopen])
        .build(mock_context(noop_assets()))
}

pub fn main() {
    let addr = mini_app::libc_calls::fopen as *const ();
    let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let options = get_options(COMMAND_NAME, fuzz_dir);
    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _res = invoke_command_minimal(app, create_payload(input.bytes()));
        ExitKind::Ok
    };
    fuzzer::main(harness, options, addr as usize);
}

fn create_payload(_bytes: &[u8]) -> InvokePayload {
    let mut args = CommandArgs::new();
    args.insert("filename", "/tmp/foo");
    args.insert("mode", "w");
    create_invoke_payload(COMMAND_NAME, args)
}

#[cfg(test)]
mod test {
    use super::*;

    // This is a trick to capture the fuzzer exit status code.
    // The fuzzer exits the process with an error code rather than panicking.
    // This test will be started as a new process and its exit status will be captured.
    #[test]
    #[ignore]
    fn real_test_fopen() {
        let addr = mini_app::libc_calls::fopen as *const ();
        let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        let options = get_options(COMMAND_NAME, fuzz_dir);
        let harness = |input: &BytesInput| {
            let app = setup_tauri_mock().expect("Failed to init Tauri app");
            let _res = invoke_command_minimal(app, create_payload(input.bytes()));
            ExitKind::Ok
        };
        unsafe {
            let _ = fuzzer::fuzz_test(harness, &options, addr as usize).is_ok();
        }
    }

    // Start another process that will actually launch the fuzzer
    #[test]
    fn test_fopen() {
        let exe = std::env::current_exe().expect("Failed to extract current executable");
        let status = std::process::Command::new(exe)
            .args(&["--ignored", "real_test_fopen"])
            .status()
            .expect("Unable to run program");

        assert_eq!(Some(134), status.code());
    }
}
