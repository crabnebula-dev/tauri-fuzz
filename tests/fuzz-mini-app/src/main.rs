use fuzzer::tauri_utils::invoke_command_minimal;
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
mod utils;
use utils::*;

const COMMAND_NAME: &str = "tauri_cmd_1";
const COMMAND_PTR: *const () = mini_app::basic::tauri_cmd_1 as *const ();

pub fn main() {
    let w = setup_mock();
    let harness = |random_input: &BytesInput| {
        let request =
            create_invoke_request_with_input_as_string(COMMAND_NAME, random_input.bytes());
        invoke_command_minimal(w.clone(), request);
        ExitKind::Ok
    };
    let options =
        fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config(), COMMAND_NAME, fuzz_dir()).into();
    fuzzer::fuzz_main(
        harness,
        &options,
        COMMAND_PTR as usize,
        policies::no_policy(),
        true,
    );
}
