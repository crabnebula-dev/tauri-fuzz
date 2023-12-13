use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;
use tauri_fuzz_tools::{
    create_invoke_payload, fuzzer, get_options, invoke_command_minimal, mock_builder_minimal,
    CommandArgs,
};

const COMMAND_NAME: &str = "write_to_stdout";

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder_minimal()
        .invoke_handler(tauri::generate_handler![
            mini_app::direct_syscalls::write_to_stdout
        ])
        .build(mock_context(noop_assets()))
}

pub fn main() {
    let options = get_options("mini_app", COMMAND_NAME);
    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _res = invoke_command_minimal(app, create_payload(input.bytes()));
        ExitKind::Ok
    };

    fuzzer::main(harness, options);
}

fn create_payload(bytes: &[u8]) -> InvokePayload {
    let input = String::from_utf8_lossy(bytes).to_string();
    let arg_name = String::from("s");
    let mut args = CommandArgs::new();
    args.insert(arg_name, input);
    create_invoke_payload(COMMAND_NAME, args)
}
