#![allow(unused_variables)]
use fuzzer::tauri_utils::invoke_command_minimal;
use libafl::inputs::BytesInput;
use libafl::prelude::ExitKind;
mod utils;
use utils::*;

const COMMAND_NAME: &str = "ls_with_rust_command_spawn";
const COMMAND_PTR: *const () = mini_app::external_process::ls_with_rust_command_spawn as *const ();

pub fn main() {
    color_backtrace::install();
    let w = setup_mock();
    let harness = |random_input: &BytesInput| {
        let request =
            // create_invoke_request_with_input_as_string(COMMAND_NAME, random_input.bytes());
            create_invoke_request_with_input_as_string(COMMAND_NAME, "lasdjfkl".as_bytes());
        invoke_command_minimal(w.clone(), request);
        ExitKind::Ok
    };
    let options =
        fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
    fuzzer::fuzz_main(
        harness,
        &options,
        COMMAND_PTR as usize,
        // policies::external_process::block_on_entry(vec!["ls".to_string()]),
        policies::external_process::block_on_rust_api_error_status(),
        true,
    );
}
