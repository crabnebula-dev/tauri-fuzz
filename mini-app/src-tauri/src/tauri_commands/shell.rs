#![allow(unused_imports)]
use crate::payload_for_tauri_cmd;
use log::trace;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use tauri::api::process::{Command, Output};
use tauri::api::shell::Program;
use tauri::hooks::InvokePayload;

#[tauri::command]
pub fn open_command(input: String) {
    trace!("[open_injection] Entering with input: {:?}", input);
    unimplemented!()
}

#[tauri::command]
pub fn shell_command_0(input: String) -> tauri::Result<String> {
    trace!("[shell_command_0] Entering with input: {:?}", input);
    res_output_to_tauri_res(Command::new(input).output())
}

#[tauri::command]
pub fn shell_command_1(input: String, arg: String) -> tauri::Result<String> {
    trace!(
        "[shell_command_1] Entering with input: {:?} and arg: [{:?}]",
        input,
        arg
    );
    res_output_to_tauri_res(Command::new(input).args([arg]).output())
}

#[tauri::command]
pub fn bin_sh(input: String) -> tauri::Result<String> {
    trace!("[bin_sh] Entering with input: {:?}", input,);
    res_output_to_tauri_res(Command::new("/bin/sh").args(["-c", &input]).output())
}

pub fn payload_for_shell_command_0(bytes: &[u8]) -> InvokePayload {
    let input = String::from_utf8_lossy(bytes).to_string();
    let arg_name = String::from("input");
    let arg_value = JsonValue::String(input);
    let mut args = HashMap::new();
    args.insert(arg_name, arg_value);
    payload_for_tauri_cmd(String::from("shell_command_0"), args)
}

pub fn payload_for_shell_command_1(bytes: &[u8], arg: String) -> InvokePayload {
    let input = String::from_utf8_lossy(bytes).to_string();
    let arg_name = String::from("input");
    let arg_value = JsonValue::String(input);
    let mut args = HashMap::new();
    args.insert(arg_name, arg_value);
    args.insert(String::from("arg"), JsonValue::String(arg));
    payload_for_tauri_cmd(String::from("shell_command_1"), args)
}

pub fn payload_for_bin_sh(bytes: &[u8]) -> InvokePayload {
    let input = String::from_utf8_lossy(bytes).to_string();
    let arg_name = String::from("input");
    let arg_value = JsonValue::String(input);
    let mut args = HashMap::new();
    args.insert(arg_name, arg_value);
    payload_for_tauri_cmd(String::from("bin_sh"), args)
}

fn res_output_to_tauri_res(
    res: Result<tauri::api::process::Output, tauri::api::Error>,
) -> tauri::Result<String> {
    res.map(|output| output.stdout)
        .map_err(|api_err| tauri::error::Error::FailedToExecuteApi(api_err))
}
