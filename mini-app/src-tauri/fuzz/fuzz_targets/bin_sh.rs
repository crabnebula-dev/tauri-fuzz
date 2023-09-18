#![no_main]

use libfuzzer_sys::fuzz_target;
use mini_app::tauri_commands::shell::*;

use std::process::Command;

fuzz_target!(|data: &[u8]| {
    bin_sh(data);
});
