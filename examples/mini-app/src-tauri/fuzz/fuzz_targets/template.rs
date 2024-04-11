/// This is a template to create a fuzz target
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;
use fuzzer::tauri_utils::{create_invoke_payload, invoke_command_minimal, CommandArgs};

/// The name of Tauri command you want to fuzz
/// Ex: tauri_cmd_1
const COMMAND_NAME: &str = todo!();

pub fn main() {
    // Take the function pointer to the harness and send it to the fuzzer
    // The fuzzer will start its runtime analysis only while executing this harness
    let ptr = crate::harness as *const ();
    // Tell the fuzzer the path to this fuzz directory
    let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    // Set the options for fuzzing
    let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);

    // Start the fuzzer
    fuzzer::fuzz_main(
        harness,
        options,
        ptr as usize,
        // The policy we want to apply
        // Ex: policies::no_policy()
        todo!(),
    );
}

/// The harness that will be executed by the fuzzer for every input
/// The fuzzer will generate inputs as `BytesInput` and expect an `ExitKind`
fn harness(input: &BytesInput) -> ExitKind {
    // Setup a Tauri application `MockRuntime` with minimal features
    let app = mock_builder_minimal()
        // Init the Tauri application with the Tauri commands you want to invoke
        /// #CUSTOMIZE
        .invoke_handler(tauri::generate_handler![crate_name::my_tauri_command])
        .build(mock_context(noop_assets()))
        .expect("Failed to init Tauri app");

    // // If you require an application that uses a `Tauri::State` don't spawn a minimal runtime
    // let app = mock_builder()
    //     .invoke_handler(tauri::generate_handler![mini_app::basic::tauri_cmd_1])
    //     .build(mock_context(noop_assets()))
    //     .expect("Failed to init Tauri app");

    // Invoke the command you want to fuzz with `T` the type you expect in return
    let _ = invoke_command::<T>(app, create_payload(input.bytes()));

    // // If you don't expect a return value from the command you can use this
    // let _ = invoke_command_minimal(app, create_payload(input.bytes()));

    ExitKind::Ok
}

/// Helper code to create an `InvokePayload` to call your Tauri command
fn create_payload(bytes: &[u8]) -> InvokePayload {
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

    // Call a helper function to create the `InvokePayload`
    create_invoke_payload(None, COMMAND_NAME, args)
}
