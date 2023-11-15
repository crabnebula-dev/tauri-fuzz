mod frida_fuzzer;
mod fuzz_utils;
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{create_invoke_payload, CommandArgs};
use tauri::test::{invoke_command_and_stop, mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;

pub fn main() {
    let options = fuzz_utils::get_options("tauri_cmd_2", "libmini_app.so");

    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _ = invoke_command_and_stop::<String>(app, payload_for_tauri_cmd_1(input.bytes()));
        ExitKind::Ok
    };

    frida_fuzzer::main(harness, options);
}

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder()
        .invoke_handler(tauri::generate_handler![mini_app::tauri_cmd_1])
        .build(mock_context(noop_assets()))
}

// Helper code to create a payload tauri_cmd_1
fn payload_for_tauri_cmd_1(bytes: &[u8]) -> InvokePayload {
    let input = String::from_utf8_lossy(bytes).to_string();
    let arg_name = String::from("input");
    let mut args = CommandArgs::new();
    args.insert(arg_name, input);
    create_invoke_payload(String::from("tauri_cmd_1"), args)
}
