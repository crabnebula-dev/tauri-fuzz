// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(unused_imports)]
#[cfg(not(target_os = "windows"))]
pub use os_not_windows::*;
#[cfg(target_os = "windows")]
pub use os_windows::*;
use std::collections::HashMap;
use tracing::info;

#[cfg(not(target_os = "windows"))]
mod os_not_windows {
    use super::*;

    #[tauri::command]
    pub fn ls_with_rust_command_output(input: String) -> String {
        info!("[ls_with_rust_command] Entering with input: {:?}", input);
        let mut ls = std::process::Command::new("ls");
        let output = ls.arg(&input).output().unwrap_or_else(|_| {
            panic!("Failed [ls_with_rust_command_output] command with input {input}")
        });
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    #[tauri::command]
    pub fn ls_with_rust_command_status(input: String) -> Option<i32> {
        info!("[ls_with_rust_command] Entering with input: {:?}", input);
        let mut ls = std::process::Command::new("ls");
        let status = ls.arg(&input).status().unwrap_or_else(|_| {
            panic!("Failed [ls_with_rust_command_status] command with input {input}")
        });
        status.code()
    }

    #[tauri::command]
    pub fn ls_with_rust_command_spawn(input: String) -> Option<i32> {
        info!("[ls_with_rust_command] Entering with input: {:?}", input);
        let mut ls = std::process::Command::new("ls")
            .arg(&input)
            .spawn()
            .unwrap_or_else(|_| {
                panic!("Failed [ls_with_rust_command_spawn] command with input {input}")
            });
        let output = ls.wait().unwrap_or_else(|_| {
            panic!("Failed [ls_with_rust_command_spawn] command with input {input}",)
        });
        output.code()
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
            .arg(format!("ls {input}"))
            .output()
            .unwrap_or_else(|_| panic!("Failed [ls_with_shell] command with input {input}"));
        String::from_utf8_lossy(&output.stdout).to_string()
    }
}

#[cfg(target_os = "windows")]
mod os_windows {
    use super::*;

    #[tauri::command]
    pub fn ls_with_rust_command_output(input: String) -> String {
        info!("[ls_with_rust_command] Entering with input: {:?}", input);
        let mut ls = std::process::Command::new("cmd");
        ls.args(["/C", "dir", &input]);
        let output = ls.output().unwrap_or_else(|_| {
            panic!("Failed [ls_with_rust_command_output] command with input {input}")
        });
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    #[tauri::command]
    pub fn ls_with_rust_command_status(input: String) -> Option<i32> {
        info!("[ls_with_rust_command] Entering with input: {:?}", input);
        let mut ls = std::process::Command::new("cmd");
        ls.args(["/C", "dir", &input]);
        let status = ls.status().unwrap_or_else(|_| {
            panic!("Failed [ls_with_rust_command_status] command with input {input}")
        });
        status.code()
    }

    #[tauri::command]
    pub fn ls_with_rust_command_spawn(input: String) -> Option<i32> {
        info!("[ls_with_rust_command] Entering with input: {:?}", input);
        let mut ls = std::process::Command::new("cmd");
        ls.args(["/C", "dir", &input]);
        let mut ls = ls.spawn().unwrap_or_else(|_| {
            panic!("Failed [ls_with_rust_command_spawn] command with input {input}")
        });
        let output = ls.wait().unwrap_or_else(|_| {
            panic!("Failed [ls_with_rust_command_spawn] command with input {input}")
        });
        output.code()
    }

    #[tauri::command]
    pub fn ls_with_tauri_plugin(input: String) -> String {
        info!("[ls_with_tauri_plugin] Entering with input: {:?}", input);
        unimplemented!()
    }

    #[tauri::command]
    pub fn ls_with_shell(input: String) -> String {
        info!("[ls_with_shell] Entering with input: {:?}", input);
        unimplemented!()
    }
}
