#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![allow(unused_imports)]

use crate::{bytes_input_to_u32, payload_for_tauri_cmd};
use log::trace;
use serde_json::{Number, Value as JsonValue};
use std::collections::HashMap;
use tauri::hooks::InvokePayload;

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

// Helper code to create a payload tauri_cmd_1
pub fn payload_for_tauri_cmd_1(bytes: &[u8]) -> InvokePayload {
    let input = String::from_utf8_lossy(bytes).to_string();
    let arg_name = String::from("input");
    let arg_value = JsonValue::String(input);
    let mut args = HashMap::new();
    args.insert(arg_name, arg_value);
    payload_for_tauri_cmd(String::from("tauri_cmd_1"), args)
}

// Helper code to create a payload tauri_cmd_2
pub fn payload_for_tauri_cmd_2(bytes: &[u8]) -> InvokePayload {
    let input = bytes_input_to_u32(bytes);
    let arg_name = String::from("input");
    let arg_value = JsonValue::Number(Number::from(input));
    let mut args = HashMap::new();
    args.insert(arg_name, arg_value);
    payload_for_tauri_cmd(String::from("tauri_cmd_2"), args)
}
