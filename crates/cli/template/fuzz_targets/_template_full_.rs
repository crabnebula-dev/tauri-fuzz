/// This is a template to create a fuzz target
///
/// Steps:
/// 1. Copy this file and rename it
/// 2. Change `COMMAND_NAME` const value on line 17
/// 3. Change the path to your command in `tauri::generate_handler` on line 34
/// 4. Modify `create_payload` to create arguments for your command on line 43
/// 5. Finally add the new fuzz target in [[bin]] table in Cargo.toml of your project
///
use fuzzer::tauri_utils::{create_invoke_payload, invoke_command_minimal, CommandArgs};
use fuzzer::SimpleFuzzerConfig;
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::InvokePayload;

const COMMAND_NAME: &str = "greet";

fn main() {
    let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let fuzz_config_file = fuzz_dir.join("fuzzer_config.toml");
    let options = SimpleFuzzerConfig::from_toml(fuzz_config_file, COMMAND_NAME, fuzz_dir).into();
    fuzzer::fuzz_main(
        harness,
        options,
        harness as *const () as usize,
        policies::file_policy::no_file_access(),
    );
}

fn harness(input: &BytesInput) -> ExitKind {
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![{{crate_name_underscored}}::greet])
        .build(mock_context(noop_assets()))
        .expect("Failed to init Tauri app");

    let _ = invoke_command_minimal(app, create_payload(input.bytes()));

    ExitKind::Ok
}

fn create_payload(bytes: &[u8]) -> InvokePayload {
    let mut params = CommandArgs::new();

    let param = String::from_utf8_lossy(&bytes).to_string();
    params.insert("name", param);

    create_invoke_payload(None, COMMAND_NAME, params)
}
