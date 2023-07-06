//! A libfuzzer-like fuzzer using qemu for binary-only coverage
#[cfg(target_os = "linux")]
mod fuzzer;
mod utils;
mod qemu;

#[cfg(target_os = "linux")]
pub fn main() {
    fuzzer::fuzz();
}

#[cfg(not(target_os = "linux"))]
pub fn main() {
    panic!("qemu-user and libafl_qemu is only supported on linux!");
}
