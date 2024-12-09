// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(unused_imports)]
pub mod tauri_commands;
use tauri::{
    test::{mock_builder, mock_context, noop_assets},
    WebviewWindow,
};
pub use tauri_commands::basic;
pub use tauri_commands::demo;
pub use tauri_commands::external_process;
pub use tauri_commands::file_access;
pub use tauri_commands::libc_calls;
pub use tauri_commands::sql;
use tracing::info;
use tracing_subscriber::fmt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let format = fmt::format().pretty();
    tracing_subscriber::fmt::fmt().event_format(format).init();

    info!("Start tracing");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            crate::file_access::write_foo_file,
            crate::file_access::read_foo_file,
            crate::basic::tauri_cmd_2,
            crate::basic::tauri_cmd_1,
            crate::external_process::ls_with_rust_command_output,
            crate::external_process::ls_with_shell,
        ])
        .setup(move |app| {
            tauri::WebviewWindowBuilder::new(app, "main", Default::default())
                .build()
                .unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Failed to init Tauri app");
}

// fn start_graphical() {
//     app.run(|_, _| {});
// }
//
// fn with_mockruntime() {}
