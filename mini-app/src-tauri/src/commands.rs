#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![allow(unused_imports)]

use log::trace;

#[tauri::command]
pub fn tauri_cmd_1(input: &str) -> String {
    trace!("[tauri_cmd_1] Entering with input: {}", input);
    let mut bytes = input.bytes();
    if bytes.next() == Some(b'a') {
        if bytes.next() == Some(b'b') {
            if bytes.next() == Some(b'c') {
                panic!("[mini-app] Crashing! =)");
            }
        }
    }
    format!("Hello, you wrote {}!", input)
}

#[tauri::command]
pub fn tauri_cmd_2(input: u32) -> String {
    trace!("[tauri_cmd_2] Entering with input: {}", input);
    if input == 100 {
        panic!("[mini-app] Crashing! =)");
    }
    format!("Hello, you wrote {}!", input)
}
