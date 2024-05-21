use fuzzer::tauri_utils::{create_invoke_request, invoke_command_minimal, CommandArgs};
/// This is a template to create a fuzz target
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::webview::InvokeRequest;
use tauri::App as TauriApp;

/// The name of Tauri command you want to fuzz
/// Ex: tauri_cmd_1
const COMMAND_NAME: &str = todo!();
/// Pointer to the Tauri command you want to fuzz
const COMMAND_PTR: *const () = todo!();

fn setup_mock() -> tauri::WebviewWindow<MockRuntime> {
    // Setup a Tauri application `MockRuntime`
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![todo!()])
        .build(mock_context(noop_assets()))
        .expect("Failed to init Tauri app");
    // Attach a main window to the app
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    webview
}

pub fn main() {
    // Set the options for the fuzzer
    let options =
        fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();

    // Setup the Tauri MockRuntime and get the main webview
    let w = setup_mock();

    // The harness that will be executed by the fuzzer for every input
    // The fuzzer will generate inputs as `BytesInput` and expect an `ExitKind`
    let harness = |input: &BytesInput| {
        invoke_command_minimal(w.clone(), create_request(input.bytes()));
        ExitKind::Ok
    };

    // Start the fuzzer
    fuzzer::fuzz_main(
        harness,
        options,
        COMMAND_PTR as usize,
        // The policy we want to apply
        // Ex: policies::no_policy()
        todo!(),
    );
}

/// Helper code to create an `InvokeRequest` to call your Tauri command
fn create_request(bytes: &[u8]) -> InvokeRequest {
    // This function needs to be customized depending on the tauri command invoked
    // The code below is an example of a tauri command that takes a `String` parameter
    // that is names "param1".
    todo!();

    let string_input = String::from_utf8_lossy(bytes).to_string();

    // Prepare the parameters of the Tauri command
    let mut params = CommandArgs::new();

    // Do type conversion from bytes to the parameters expected type.
    // In this example the first parameter is a `String`
    let string_input = String::from_utf8_lossy(bytes).to_string();
    // The name of Tauri command parameter with its associated value
    params.insert("param1".into(), input);

    // Call a helper function to create the `InvokeRequest`
    create_invoke_request(None, COMMAND_NAME, args)
}
