// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, you wrote {}!", name)
}
