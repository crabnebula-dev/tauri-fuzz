use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;
use tauri_fuzz_tools::{
    create_invoke_payload, fuzzer, get_options, invoke_command_and_stop, CommandArgs,
};

pub fn main() {
    let options = get_options("tauri_cmd_2", vec!["libmini_app.so"]);

    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _ = invoke_command_and_stop::<String>(app, payload_for_tauri_cmd_1(input.bytes()));
        ExitKind::Ok
    };

    fuzzer::main(harness, options);
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
    create_invoke_payload("tauri_cmd_1", args)
}
