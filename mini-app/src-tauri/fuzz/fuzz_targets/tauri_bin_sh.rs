#![no_main]

use libfuzzer_sys::fuzz_target;
use mini_app::tauri_commands::shell::*;
use mini_app::*;

fuzz_target!(|data: &[u8]| {
    let app = setup_tauri_mock().expect("Failed to init Tauri app");
    call_tauri_cmd(app, payload_for_bin_sh(data));
});
