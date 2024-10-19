// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use libafl::inputs::{BytesInput, HasMutatorBytes};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::webview::InvokeRequest;
use tauri::WebviewWindow;
/// This is a template to create a fuzz target
///
/// Steps:
/// 1. Copy this file and rename it
/// 2. Change `COMMAND_NAME` const value on line 23
/// 3. Change the path to your command in `tauri::generate_handler` on line 41
/// 4. Modify `create_request` to create arguments for your command on line 59
/// 5. Finally add the new fuzz target in [[bin]] table in Cargo.toml of your project
///
/// Note: you may need to implement [FromRandomBytes] for your command argument types.
///
use tauri_fuzz::tauri::{
    create_invoke_request, invoke_command_minimal, CommandArgs, FromRandomBytes,
};
use tauri_fuzz::SimpleFuzzerConfig;

const COMMAND_NAME: &str = "read_foo_file";

fn main() {
    let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let fuzz_config_file = fuzz_dir.join("fuzzer_config.toml");
    let options = SimpleFuzzerConfig::from_toml(fuzz_config_file, COMMAND_NAME, fuzz_dir).into();
    tauri_fuzz::fuzz_main(
        harness,
        &options,
        harness as *const () as usize,
        tauri_fuzz_policies::filesystem::no_file_access(),
        false,
    );
}

// Setup the Tauri application mockruntime and an associated "main" webview
fn setup_mock() -> WebviewWindow<MockRuntime> {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            mini_app::file_access::read_foo_file
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
fn harness(input: &BytesInput) -> ExitKind {
    let webview = setup_mock();
    let _ = invoke_command_minimal(webview, create_request(input.bytes()));
    ExitKind::Ok
}

// Helper code to create an `InvokeRequest` to send to the Tauri app backend
fn create_request(bytes: &[u8]) -> InvokeRequest {
    let mut params = CommandArgs::new();

    let param = String::from_random_bytes(&bytes).unwrap();
    params.insert("name", param);

    create_invoke_request(None, COMMAND_NAME, params)
}
