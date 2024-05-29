#![allow(unused_imports)]
pub mod tauri_commands;
pub use tauri_commands::basic;
pub use tauri_commands::file_access;
pub use tauri_commands::libc_calls;
// pub use tauri_commands::shell::*;
use fuzzer::tauri_utils::{create_invoke_request, invoke_command, CommandArgs};
use tauri::{
    test::{mock_builder, mock_context, noop_assets},
    WebviewWindow,
};
pub use tauri_commands::demo;
pub use tauri_commands::sql;
use tracing::info;
use tracing_subscriber::fmt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let format = fmt::format().pretty();
    tracing_subscriber::fmt::fmt().event_format(format).init();
    // tracing_subscriber::registry()
    //     .with(fmt::layer())
    //     .with(EnvFilter::from_default_env())
    //     .init();

    info!("Start tracing");

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            crate::file_access::write_foo_file,
            crate::file_access::read_foo_file,
            crate::basic::tauri_cmd_2,
            crate::basic::tauri_cmd_1
        ])
        .setup(move |app| {
            tauri::WebviewWindowBuilder::new(app, "main", Default::default())
                .build()
                .unwrap();
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Failed to init Tauri app");
    app.run(|_, _| {});

    // call_fs_readFile(webview);

    // let mut args = CommandArgs::new();
    // let mut path = std::env::current_dir().unwrap();
    // path.push("test_assets");
    // path.push("foo.txt");
    // args.insert("path", path.to_str().unwrap());
    //
    // let payload = create_invoke_request(None, "read_foo_file", args);
    //
    // let res = invoke_command::<String, String>(&webview, payload);
    // println!("{:?}", res);
}
