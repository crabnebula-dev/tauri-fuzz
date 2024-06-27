// A module to write Tauri commands to test the fuzzer
pub mod basic;
pub mod demo;
pub mod file_access;
pub mod libc_calls;
#[cfg(unix)]
pub mod shell;
pub mod sql;
