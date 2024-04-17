use fuzzer::tauri_utils::{create_invoke_payload, invoke_command_minimal, CommandArgs};
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;

const COMMAND_NAME: &str = "sql_transaction";

pub fn main() {
    let ptr = mini_app::sql::sql_transaction as *const ();
    let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _ = invoke_command_minimal(app, create_payload(input.bytes()));
        ExitKind::Ok
    };

    fuzzer::fuzz_main(harness, options, ptr as usize, policies::no_policy());
}

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder()
        .invoke_handler(tauri::generate_handler![mini_app::sql::sql_transaction])
        .build(mock_context(noop_assets()))
}

// Helper code to create a payload for sql_transaction
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
    #[ignore] // We currently ignore the test since it requires an initialization of the SQL
              // database TODO
    fn no_crash_sql_transaction() {
        let addr = mini_app::sql::sql_transaction as *const ();
        let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
        let harness = |_input: &BytesInput| {
            let app = setup_tauri_mock().expect("Failed to init Tauri app");
            let _res = invoke_command_minimal(app, create_payload("foo".as_bytes()));
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
    fn hidden_crash_sql_transaction() {
        let addr = mini_app::sql::sql_transaction as *const ();
        let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
        let harness = |_input: &BytesInput| {
            let app = setup_tauri_mock().expect("Failed to init Tauri app");
            let _res = invoke_command_minimal(app, create_payload("a'".as_bytes()));
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
    #[ignore] // We currently ignore the test since it requires an initialization of the SQL
              // database TODO
    fn crash_sql_transaction() {
        let exe = std::env::current_exe().expect("Failed to extract current executable");
        let status = std::process::Command::new(exe)
            .args(&["--ignored", "hidden_crash_sql_transaction"])
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
