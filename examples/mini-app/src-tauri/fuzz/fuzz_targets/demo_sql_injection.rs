use fuzzer::tauri_utils::{create_invoke_payload, invoke_command_minimal, CommandArgs};
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;
mod utils;
use utils::*;

const COMMAND_NAME: &str = "sql_injection_vulnerability";

pub fn main() {
    let ptr = mini_app::demo::sql_injection_vulnerability as *const ();
    let options = fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        invoke_command_minimal(app, create_payload(input.bytes()));
        ExitKind::Ok
    };

    fuzzer::fuzz_main(harness, options, ptr as usize, policies::no_policy());
}

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder()
        .invoke_handler(tauri::generate_handler![
            mini_app::demo::sql_injection_vulnerability
        ])
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
