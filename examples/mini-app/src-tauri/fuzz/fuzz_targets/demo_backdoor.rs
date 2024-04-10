use fuzzer::tauri_utils::{
    create_invoke_payload, invoke_command_minimal, mock_builder_minimal, CommandArgs,
};
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;

const COMMAND_NAME: &str = "tauri_cmd_with_backdoor";

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder_minimal()
        .invoke_handler(tauri::generate_handler![
            mini_app::tauri_commands::demo::tauri_cmd_with_backdoor
        ])
        .build(mock_context(noop_assets()))
}

pub fn main() {
    println!("Starting demo...");
    let addr = mini_app::tauri_commands::demo::tauri_cmd_with_backdoor as *const () as usize;
    let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);

    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _res = invoke_command_minimal(app, create_payload(input.bytes()));
        ExitKind::Ok
    };

    println!("Starting the fuzzer...");
    fuzzer::fuzz_main(
        harness,
        options,
        addr,
        policies::file_policy::no_file_access(),
    );
}

// Helper code to create a payload for `tauri_cmd_with_backdoor`
fn create_payload(bytes: &[u8]) -> InvokePayload {
    let input = bytes_input_to_u32(bytes);
    let arg_name = String::from("input");
    let mut args = CommandArgs::new();
    args.insert(arg_name, input);
    create_invoke_payload(None, COMMAND_NAME, args)
}

fn bytes_input_to_u32(bytes_input: &[u8]) -> u32 {
    let mut array_input = [0u8; 4];
    for (dst, src) in array_input.iter_mut().zip(bytes_input) {
        *dst = *src
    }
    let res = u32::from_be_bytes(array_input);
    res
}
