mod frida_fuzzer;
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use libafl_bolts::bolts_prelude::Cores;
use libafl_bolts::cli::FuzzerOptions;
use std::path::PathBuf;
use std::str::FromStr;
use tauri::fuzz::{create_invoke_payload, CommandArgs};
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;

pub fn main() {
    let options = FuzzerOptions {
        timeout: std::time::Duration::from_secs(5),
        verbose: true,
        stdout: String::from("/dev/stdout"),
        configuration: String::from("default configuration"),
        asan: true,
        asan_cores: Cores::from_cmdline("0").unwrap(),
        iterations: 0,
        harness: Some(PathBuf::from_str("tauri_cmd_1").unwrap()),
        harness_args: vec![],
        harness_function: String::from(""),
        libs_to_instrument: vec![],
        cmplog: true,
        cmplog_cores: Cores::from_cmdline("0").unwrap(),
        detect_leaks: false,
        continue_on_error: false,
        allocation_backtraces: true,
        max_allocation: 1073741824,
        max_total_allocation: 4294967296,
        max_allocation_panics: true,
        disable_coverage: false,
        drcov: false,
        disable_excludes: true,  // check
        dont_instrument: vec![], // check
        tokens: vec![],          // check
        // input: vec![PathBuf::from_str("tauri_cmd_1_fuzz/corpus").unwrap()],
        input: vec![],
        output: PathBuf::from_str("tauri_cmd_1_solutions").unwrap(),
        // Doesn't work on MacOS
        // cores: Cores::from_cmdline("0").unwrap(),
        cores: Cores::from_cmdline("1-4").unwrap(),
        broker_port: 8888,
        remote_broker_addr: None,
        replay: None, // check
        repeat: None,
    };

    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        tauri::fuzz::invoke_tauri_cmd(app, payload_for_tauri_cmd_1(input.bytes()));
        ExitKind::Ok
    };

    frida_fuzzer::main(harness, options);
}

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder()
        .invoke_handler(tauri::generate_handler![mini_app::tauri_cmd_1])
        .build(mock_context(noop_assets()))
}

// Helper code to create a payload tauri_cmd_1
fn payload_for_tauri_cmd_1(bytes: &[u8]) -> InvokePayload {
    let input = String::from_utf8_lossy(bytes).to_string();
    let arg_name = String::from("input");
    let mut args = CommandArgs::new();
    args.insert(arg_name, input);
    create_invoke_payload(String::from("tauri_cmd_1"), args)
}
