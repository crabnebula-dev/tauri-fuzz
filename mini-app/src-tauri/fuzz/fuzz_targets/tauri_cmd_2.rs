use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;
use tauri_fuzz_tools::{create_invoke_payload, invoke_command_minimal, CommandArgs};

const COMMAND_NAME: &str = "tauri_cmd_2";

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder()
        .invoke_handler(tauri::generate_handler![mini_app::basic::tauri_cmd_2])
        .build(mock_context(noop_assets()))
}

pub fn main() {
    let addr = mini_app::basic::tauri_cmd_2 as *const ();
    let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _res = invoke_command_minimal(app, create_payload(input.bytes()));
        ExitKind::Ok
    };
    fuzzer::main(harness, options, addr as usize);
}

// Helper code to create a payload tauri_cmd_2
fn create_payload(bytes: &[u8]) -> InvokePayload {
    let input = bytes_input_to_u32(bytes);
    let arg_name = String::from("input");
    let mut args = CommandArgs::new();
    args.insert(arg_name, input);
    create_invoke_payload(COMMAND_NAME, args)
}

fn bytes_input_to_u32(bytes_input: &[u8]) -> u32 {
    let mut array_input = [0u8; 4];
    for (dst, src) in array_input.iter_mut().zip(bytes_input) {
        *dst = *src
    }
    let res = u32::from_be_bytes(array_input);
    res
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tauri_cmd_2() {
        let addr = mini_app::basic::tauri_cmd_2 as *const ();
        let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
        let harness = |input: &BytesInput| {
            let app = setup_tauri_mock().expect("Failed to init Tauri app");
            let _res = invoke_command_minimal(app, create_payload(input.bytes()));
            ExitKind::Ok
        };
        unsafe {
            assert!(fuzzer::fuzz_test(harness, &options, addr as usize).is_ok());
        }
    }
}
