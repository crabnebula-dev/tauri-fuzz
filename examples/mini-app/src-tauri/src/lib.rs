// // Copyright 2024-2022 CrabNebula Ltd., Alexandre Dang
// // SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(unused_imports)]
pub mod tauri_commands;
use fuzzer::tauri::{create_invoke_request, invoke_command, CommandArgs};
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

    // let app = mock_builder()
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

    // let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
    //     .build()
    //     .unwrap();
    //

    // call_fs_readFile(webview);

    // let mut args = CommandArgs::new();
    // args.insert("input", "-la");
    // let payload = create_invoke_request(None, "ls_with_rust_command", args);
    // let res = invoke_command::<String, String>(&webview, payload);
    // println!("{:#?}", res);
    //
    // let mut args = CommandArgs::new();
    // args.insert("input", "-la");
    // let payload = create_invoke_request(None, "ls_with_shell", args);
    // let res = invoke_command::<String, String>(&webview, payload);
    // println!("{:#?}", res);

    // path.push("test_assets");
    // path.push("foo.txt");
    // args.insert("path", path.to_str().unwrap());
    //
    //
    // let res = invoke_command::<String, String>(&webview, payload);
}

// fn start_graphical() {
//     app.run(|_, _| {});
// }
//
// fn with_mockruntime() {}
