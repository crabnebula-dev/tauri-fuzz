use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;
use tauri_fuzz_tools::{
    create_invoke_payload, fuzzer, get_options, invoke_command_minimal, mock_builder_minimal,
    CommandArgs,
};

const COMMAND_NAME: &str = "read_foo_file";

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder_minimal()
        .invoke_handler(tauri::generate_handler![
            mini_app::tauri_commands::file_access::read_foo_file
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

fn create_payload(_bytes: &[u8]) -> InvokePayload {
    let arg_name = String::from("path");
    let mut args = CommandArgs::new();
    let mut path = std::env::current_dir().unwrap();
    path.pop();
    path.push("test_assets");
    path.push("foo.txt");
    args.insert(arg_name, path.to_str().unwrap());
    create_invoke_payload(COMMAND_NAME, args)
}
