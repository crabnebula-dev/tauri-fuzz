mod frida_fuzzer;
mod fuzz_utils;
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{create_invoke_payload, CommandArgs};
use tauri::test::{invoke_command_and_stop, mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder()
        .invoke_handler(tauri::generate_handler![mini_app::tauri_cmd_2])
        .build(mock_context(noop_assets()))
}

pub fn main() {
    let options = fuzz_utils::get_options("tauri_cmd_2", "libmini_app.so");
    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _res = invoke_command_and_stop::<String>(app, payload_for_tauri_cmd_2(input.bytes()));
        ExitKind::Ok
    };

    frida_fuzzer::main(harness, options);
}

// Helper code to create a payload tauri_cmd_2
fn payload_for_tauri_cmd_2(bytes: &[u8]) -> InvokePayload {
    let input = bytes_input_to_u32(bytes);
    let arg_name = String::from("input");
    let mut args = CommandArgs::new();
    args.insert(arg_name, input);
    create_invoke_payload(String::from("tauri_cmd_2"), args)
}

fn bytes_input_to_u32(bytes_input: &[u8]) -> u32 {
    let mut array_input = [0u8; 4];
    for (dst, src) in array_input.iter_mut().zip(bytes_input) {
        *dst = *src
    }
    let res = u32::from_be_bytes(array_input);
    res
}
