//! A libfuzzer-like fuzzer using qemu for binary-only coverage
#[cfg(target_os = "linux")]
mod fuzzer;
#[cfg(feature = "qemu")]
mod qemu;
mod tauri_fuzz_tools;
mod utils;

#[cfg(target_os = "linux")]
pub fn main() {
    fuzzer::inprocess_fuzz();
}

#[cfg(not(target_os = "linux"))]
pub fn main() {
    panic!("qemu-user and libafl_qemu is only supported on linux!");
}
