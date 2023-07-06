use libafl::inputs::{HasBytesVec, BytesInput};
use std::path::PathBuf;
use std::ffi::OsStr;

pub(crate) fn bytes_input_to_u32(bytes_input: &BytesInput) -> u32 {
    let mut array_input = [0u8; 4];
    for (dst, src) in array_input.iter_mut().zip(bytes_input.bytes()) {
        *dst = *src
    }
    u32::from_be_bytes(array_input)
}

// TODO it's really bad
pub(crate) fn mini_app_path() -> PathBuf {
    // Get the target file path
    let mut mini_app_path = std::env::current_exe().unwrap();
    loop {
        if mini_app_path.file_name() == Some(&OsStr::new("fuzzer")) {
            break;
        }
        mini_app_path.pop();
    }
    mini_app_path.pop();
    mini_app_path.push(String::from("mini-app"));
    mini_app_path.push(String::from("src-tauri"));
    mini_app_path.push(String::from("target"));
    mini_app_path.push(String::from("debug"));
    mini_app_path.push(String::from("mini-app"));
    mini_app_path
}

