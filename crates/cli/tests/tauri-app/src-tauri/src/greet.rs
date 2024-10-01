// // Copyright 2024-2022 CrabNebula Ltd.
// // SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, you wrote {}!", name)
}
