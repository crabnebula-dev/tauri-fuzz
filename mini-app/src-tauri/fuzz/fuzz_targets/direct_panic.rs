use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;
use tauri_fuzz_tools::{
    create_invoke_payload, fuzzer, get_options, invoke_command_minimal, CommandArgs,
};

const COMMAND_NAME: &str = "direct_panic";

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder()
        .invoke_handler(tauri::generate_handler![mini_app::direct_panic])
        .build(mock_context(noop_assets()))
}

pub fn main() {
    let addr = mini_app::direct_panic as *const () as usize;
    let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let options = get_options(COMMAND_NAME, fuzz_dir);
    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _res = invoke_command_minimal(app, create_payload(input.bytes()));
        ExitKind::Ok
    };
    fuzzer::main(harness, options, addr);
}

#[allow(unused_variables)]
fn create_payload(bytes: &[u8]) -> InvokePayload {
    let args = CommandArgs::new();
    create_invoke_payload(COMMAND_NAME, args)
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO: We are not able to catch crash yet because
    // the fuzzer transform panic into `libc::exit`
    // #[test]
    fn test_direct_panic() {
        let old_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            return;
        }));

        let addr = mini_app::direct_panic as *const ();
        let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        let options = get_options(COMMAND_NAME, fuzz_dir);
        let harness = |input: &BytesInput| {
            let app = setup_tauri_mock().expect("Failed to init Tauri app");
            let _res = invoke_command_minimal(app, create_payload(input.bytes()));
            ExitKind::Ok
        };
        unsafe {
            fuzzer::fuzz_test(harness, &options, addr as usize).is_ok();
        }
    }
}
