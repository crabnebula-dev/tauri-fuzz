use fuzzer::tauri_utils::{create_invoke_request, invoke_command_minimal, CommandArgs};
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::webview::InvokeRequest;
mod utils;
use utils::*;

const COMMAND_NAME: &str = "tauri_cmd_with_backdoor";
const COMMAND_PTR: *const () = mini_app::demo::tauri_cmd_with_backdoor as *const ();

fn setup_mock() -> tauri::WebviewWindow<MockRuntime> {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            mini_app::demo::tauri_cmd_with_backdoor
        ])
        .build(mock_context(noop_assets()))
        .expect("Failed to init Tauri app");
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    webview
}

pub fn main() {
    println!("Starting demo...");
    let options =
        fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
    let w = setup_mock();
    let harness = |input: &BytesInput| {
        invoke_command_minimal(w.clone(), create_request(input.bytes()));
        ExitKind::Ok
    };

    println!("Starting the fuzzer...");
    fuzzer::fuzz_main(
        harness,
        options,
        COMMAND_PTR as usize,
        policies::file_policy::no_file_access(),
    );
}

// Helper code to create a payload for `tauri_cmd_with_backdoor`
fn create_request(bytes: &[u8]) -> InvokeRequest {
    let input = bytes_input_to_u32(bytes);
    let arg_name = String::from("input");
    let mut args = CommandArgs::new();
    args.insert(arg_name, input);
    create_invoke_request(None, COMMAND_NAME, args)
}

fn bytes_input_to_u32(bytes_input: &[u8]) -> u32 {
    let mut array_input = [0u8; 4];
    for (dst, src) in array_input.iter_mut().zip(bytes_input) {
        *dst = *src
    }
    u32::from_be_bytes(array_input)
}
