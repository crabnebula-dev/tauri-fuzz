#![allow(unused_imports)]
use std::collections::HashMap;
use tracing::info;

#[tauri::command]
pub fn ls_with_rust_command(input: String) -> String {
    info!("[ls_with_rust_command] Entering with input: {:?}", input);
    let mut ls = std::process::Command::new("ls");
    let ptr = &ls as *const std::process::Command as usize;
    println!("ptr: {:#018x}", ptr);
    let output = ls
        .arg(&input)
        .output()
        .unwrap_or_else(|_| panic!("Failed [ls_with_rust_command] command with input {}", input));
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[tauri::command]
pub fn ls_with_tauri_plugin(input: String) -> String {
    info!("[ls_with_tauri_plugin] Entering with input: {:?}", input);
    unimplemented!()
}

#[tauri::command]
pub fn ls_with_shell(input: String) -> String {
    info!("[ls_with_shell] Entering with input: {:?}", input);
    let mut sh = std::process::Command::new("sh");
    let output = sh
        .arg("-c")
        .arg(format!("ls {}", input))
        .output()
        .unwrap_or_else(|_| panic!("Failed [ls_with_shell] command with input {}", input));
    String::from_utf8_lossy(&output.stdout).to_string()
}
