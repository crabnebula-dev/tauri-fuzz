//! Tauri commands calling libc functions directly
// use libc::FILE;
use std::ffi::CString;

#[tauri::command]
pub fn geteuid() -> u32 {
    unsafe { libc::geteuid() }
}

#[tauri::command]
pub fn fopen(filename: &str, mode: &str) {
    let filename = CString::new(filename).expect("Cstring failed");
    let mode = CString::new(mode).expect("Cstring failed");
    unsafe { libc::fopen(filename.as_ptr(), mode.as_ptr()) };
}
