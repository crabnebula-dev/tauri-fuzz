mod fuzz_utils;
mod fuzzer;
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{create_invoke_payload, CommandArgs};
use tauri::test::{invoke_command_and_stop, mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder()
        .invoke_handler(tauri::generate_handler![
            mini_app::direct_syscalls::write_to_stdout
        ])
        .build(mock_context(noop_assets()))
}

pub fn main() {
    let options = fuzz_utils::get_options("write_to_stdout", vec!["libmini_app.so"]);
    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _res = invoke_command_and_stop::<String>(app, create_payload(input.bytes()));
        ExitKind::Ok
    };

    fuzzer::main(harness, options);
}

fn create_payload(bytes: &[u8]) -> InvokePayload {
    let input = String::from_utf8_lossy(bytes).to_string();
    let arg_name = String::from("s");
    let mut args = CommandArgs::new();
    args.insert(arg_name, input);
    create_invoke_payload(String::from("write_to_stdout"), args)
}
