#![no_main]

use libfuzzer_sys::fuzz_target;
use mini_app::tauri_commands::basic::*;
use mini_app::*;

fuzz_target!(|data: &[u8]| {
    let app = setup_tauri_mock().expect("Failed to init Tauri app");
    call_tauri_cmd(app, payload_for_tauri_cmd_1(data));
});
