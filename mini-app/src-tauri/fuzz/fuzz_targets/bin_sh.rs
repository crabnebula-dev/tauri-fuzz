#![no_main]

use libfuzzer_sys::fuzz_target;

use std::process::Command;

fuzz_target!(|data: &[u8]| {
    let mut sh = Command::new("sh");
    let input = String::from_utf8_lossy(data).to_string();
    // sh.arg("-c").arg("whoami");
    sh.arg("-c").arg(&input);
});
