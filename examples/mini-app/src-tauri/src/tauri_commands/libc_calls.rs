// // Copyright 2024-2022 CrabNebula Ltd.
// // SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! Tauri commands calling libc functions directly
use std::ffi::CString;

#[cfg(unix)]
#[tauri::command]
/// Calls libc function `geteuid`
pub fn geteuid() -> u32 {
    unsafe { libc::geteuid() }
}

#[tauri::command]
/// Calls libc function `fopen`
pub fn fopen(filename: &str, mode: &str) {
    let filename = CString::new(filename).expect("Cstring failed");
    let mode = CString::new(mode).expect("Cstring failed");
    unsafe { libc::fopen(filename.as_ptr(), mode.as_ptr()) };
}
