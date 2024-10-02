// // Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// // SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(dead_code)]
#![allow(unused_imports)]
use fuzzer::tauri::{
    create_invoke_request, invoke_command, invoke_command_minimal, setup_context_with_plugin,
    CommandArgs,
};
use fuzzer::SimpleFuzzerConfig;
use libafl::executors::ExitKind;
use libafl::inputs::BytesInput;
use policies::engine::FuzzPolicy;
use std::path::PathBuf;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::webview::InvokeRequest;
use tauri_plugin_fs::FsExt;

pub fn fuzz_config() -> PathBuf {
    const CONFIG_FILE: &str = "fuzzer_config.toml";
    let mut config_file = fuzz_dir();
    config_file.push(CONFIG_FILE);
    config_file
}

pub fn fuzz_dir() -> PathBuf {
    std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
}

#[allow(dead_code)]
pub fn path_to_foo() -> PathBuf {
    let mut assets_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    assets_dir.push("tests");
    assets_dir.push("assets");
    assets_dir.push("foo.txt");
    assets_dir
}

/// Setup a `MockRuntime` with handlers to all the Tauri commands of
/// `mini-app` and access to the plugin `fs:read-files`
pub fn setup_mock() -> tauri::WebviewWindow<MockRuntime> {
    // Permission to the fs plugin to read files
    const FS_READ_FILE_PERMISSION: &str = r#"
[[permission]]
identifier = "read-files"
description = "This enables file read related commands without any pre-configured accessible paths."
commands.allow = [
    "read_file",
]"#;

    // Capability given to our mock app, use `fs:read-files` permission
    const CAPABILITY: &str = r#"{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "fs:read-files"
  ]
}"#;

    // Prepare context with right permissions and capability
    let mut context: tauri::Context<MockRuntime> = mock_context(noop_assets());
    setup_context_with_plugin(&mut context, "fs", FS_READ_FILE_PERMISSION, CAPABILITY);

    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            mini_app::basic::tauri_cmd_1,
            mini_app::basic::tauri_cmd_2,
            mini_app::basic::direct_panic,
            mini_app::libc_calls::fopen,
            mini_app::file_access::read_foo_file,
            mini_app::file_access::write_foo_file,
            mini_app::sql::sql_transaction,
            mini_app::external_process::ls_with_rust_command_status,
            mini_app::external_process::ls_with_rust_command_output,
            mini_app::external_process::ls_with_rust_command_spawn,
            mini_app::demo::tauri_cmd_with_backdoor,
            mini_app::demo::sql_injection_vulnerability,
        ])
        .plugin(tauri_plugin_fs::init())
        .build(context)
        .expect("Failed to init Tauri app");

    // Modify the scope of the fs plugin
    let scope = app.fs_scope();
    scope.allow_file(path_to_foo().to_str().unwrap());

    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    webview
}

pub fn create_invoke_request_with_input_as_string(
    command_name: &str,
    bytes: &[u8],
) -> InvokeRequest {
    let string_input = String::from_utf8_lossy(bytes).to_string();
    invoke_request_with_input(command_name, string_input)
}

pub fn create_invoke_request_with_input_as_u32(command_name: &str, bytes: &[u8]) -> InvokeRequest {
    let mut array_input = [0u8; 4];
    for (dst, src) in array_input.iter_mut().zip(bytes) {
        *dst = *src
    }
    let u32_input = u32::from_be_bytes(array_input);
    invoke_request_with_input(command_name, u32_input)
}

fn invoke_request_with_input<T>(command_name: &str, input: T) -> InvokeRequest
where
    T: serde::ser::Serialize,
{
    let arg_name = String::from("input");
    let mut args = CommandArgs::new();
    args.insert(arg_name, input);
    create_invoke_request(None, command_name, args)
}

pub fn fuzz_command_with_arg<T>(
    command_name: &str,
    command_ptr: Option<usize>,
    policy: FuzzPolicy,
    args: Vec<(&str, T)>,
    tauri_plugin: Option<String>,
) where
    T: serde::ser::Serialize + Clone,
{
    let options = SimpleFuzzerConfig::from_toml(fuzz_config(), command_name, fuzz_dir()).into();
    let webview = setup_mock();
    let monitored_code = command_ptr.unwrap_or(fuzz_harness::<T> as usize);
    fuzzer::fuzz_main(
        |input| fuzz_harness(&webview, command_name, &args, &tauri_plugin, input),
        &options,
        monitored_code,
        policy,
        true,
    )
}

pub fn fuzz_harness<T>(
    // pub fn fuzz_harness(
    webview: &tauri::WebviewWindow<MockRuntime>,
    command_name: &str,
    args: &[(&str, T)],
    // args: &[(&str, PathBuf)],
    tauri_plugin: &Option<String>,
    _input: &BytesInput,
) -> ExitKind
where
    T: serde::ser::Serialize + Clone,
{
    let mut command_args = CommandArgs::new();
    for arg in args.iter() {
        command_args.insert(arg.0, arg.1.clone());
    }
    let request = create_invoke_request(tauri_plugin.clone(), command_name, command_args);
    invoke_command_minimal(webview.clone(), request);
    // // If we want to get a response
    // let res = invoke_command::<String, String>(&webview.clone(), request);
    // println!("{:?}", res);
    ExitKind::Ok
}

