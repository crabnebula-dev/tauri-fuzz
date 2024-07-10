/// This is a template to create a fuzz target
///
/// Steps:
/// 1. Copy this file and rename it
/// 2. Change `COMMAND_NAME` const value on line 20
/// 3. Change the path to your command in `tauri::generate_handler` on line 37
/// 4. Modify `create_request` to create arguments for your command on line 55
/// 5. Finally add the new fuzz target in [[bin]] table in Cargo.toml of your project
///
/// Note: you may need to implement [FromRandomBytes] for your command argument types.
///
use fuzzer::tauri_utils::{create_invoke_request, invoke_command_minimal, CommandArgs};
use fuzzer::SimpleFuzzerConfig;
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::webview::InvokeRequest;
use tauri::WebviewWindow;

const COMMAND_NAME: &str = "ls_with_rust_command";

fn main() {
    let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let fuzz_config_file = fuzz_dir.join("fuzzer_config.toml");
    let options = SimpleFuzzerConfig::from_toml(fuzz_config_file, COMMAND_NAME, fuzz_dir).into();

    let w = setup_mock();
    let harness = |input: &BytesInput| {
        invoke_command_minimal(w.clone(), create_request(input.bytes()));
        ExitKind::Ok
    };

    fuzzer::fuzz_main(
        harness,
        &options,
        mini_app::shell::ls_with_rust_command as *const () as usize,
        policies::external_process::block_execv_on_error(),
        true,
    );
}

// Setup the Tauri application mockruntime and an associated "main" webview
fn setup_mock() -> WebviewWindow<MockRuntime> {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            mini_app::shell::ls_with_rust_command
        ])
        .build(mock_context(noop_assets()))
        .expect("Failed to init Tauri app");
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    webview
}

// Harness function that will be repeated extensively by the fuzzer with semi-random bytes
// inputs
// fn harness(input: &BytesInput) -> ExitKind {
//     let webview = setup_mock();
//     invoke_command_minimal(webview, create_request(input.bytes()));
//     ExitKind::Ok
// }

// Helper code to create an `InvokeRequest` to send to the Tauri app backend
fn create_request(bytes: &[u8]) -> InvokeRequest {
    let mut params = CommandArgs::new();

    // let param = String::from_random_bytes(bytes).unwrap();
    let param = String::from("-la");
    params.insert("input", param);

    create_invoke_request(None, COMMAND_NAME, params)
}
