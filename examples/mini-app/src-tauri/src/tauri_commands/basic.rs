// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![allow(unused_imports)]

use std::collections::HashMap;

#[tauri::command]
/// Crash on input `abc`
pub fn tauri_cmd_1(input: &str) -> String {
    tracing::debug!("[tauri_cmd_1] Entering with input: {}", input);
    let mut bytes = input.bytes();
    if bytes.next() == Some(b'a') && bytes.next() == Some(b'b') && bytes.next() == Some(b'c') {
        panic!("[mini-app] Crashing! =)");
    }
    format!("Hello, you wrote {}!", input)
}

#[tauri::command]
/// Crash on input `100`
pub fn tauri_cmd_2(input: u32) -> String {
    // log::debug!("[tauri_cmd_2] Entering with input: {}", input);
    if input == 100 {
        panic!("[mini-app] Crashing! =)");
    }
    format!("Hello, you wrote {}!", input)
}

#[tauri::command]
/// Crash automatically
pub fn direct_panic() {
    panic!("[mini-app] Crashing! =)")
}

#[tauri::command]
pub fn no_args() -> String {
    String::from("toto")
}
