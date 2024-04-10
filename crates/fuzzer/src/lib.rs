mod fuzzer;
mod fuzzer_options;
pub use crate::fuzzer::{fuzz_main, fuzz_test};
pub use crate::fuzzer_options::get_fuzzer_options;

#[cfg(feature = "tauri")]
pub mod tauri_utils;
