use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;
use tauri_fuzz_tools::{create_invoke_payload, invoke_command_minimal, CommandArgs};

const COMMAND_NAME: &str = "direct_panic";

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder()
        .invoke_handler(tauri::generate_handler![mini_app::basic::direct_panic])
        .build(mock_context(noop_assets()))
}

pub fn main() {
    let addr = mini_app::basic::direct_panic as *const () as usize;
    let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _res = invoke_command_minimal(app, create_payload(input.bytes()));
        ExitKind::Ok
    };
    fuzzer::main(harness, options, addr, fuzzer::policies::no_policy());
}

#[allow(unused_variables)]
fn create_payload(bytes: &[u8]) -> InvokePayload {
    let args = CommandArgs::new();
    create_invoke_payload(None, COMMAND_NAME, args)
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
        let addr = mini_app::basic::direct_panic as *const ();
        let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
        let harness = |input: &BytesInput| {
            let app = setup_tauri_mock().expect("Failed to init Tauri app");
            let _res = invoke_command_minimal(app, create_payload(input.bytes()));
            ExitKind::Ok
        };
        unsafe {
            let _ = fuzzer::fuzz_test(
                harness,
                &options,
                addr as usize,
                fuzzer::policies::no_policy(),
            )
            .is_ok();
        }
    }

    // Start another process that will actually launch the fuzzer
    #[test]
    fn test_direct_panic() {
        let exe = std::env::current_exe().expect("Failed to extract current executable");
        let status = std::process::Command::new(exe)
            .args(&["--ignored", "real_test_direct_panic"])
            .status()
            .expect("Unable to run program");
        
        let mut pwd = Command::new("pwd");
        let mut ls = Command::new("ls");

        let out1 = pwd.output().expect("failed to execute process");
        let out2 = ls.output().expect("failed to execute process");

        println!("{out1} \n {out2}");

        assert_eq!(Some(134), status.code());
    }
}
