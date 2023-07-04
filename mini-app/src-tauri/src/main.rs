#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![allow(unused_imports)]

#[tauri::command]
#[no_mangle]
fn tauri_cmd_1(input: &str) -> String {
    println!("[mini-app] Entering tauri_cmd_1 with input:\n{}", input);
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
#[no_mangle]
fn tauri_cmd_2(input: u32) -> String {
    println!("[mini-app] Entering tauri_cmd_2 with input:\n{}", input);
    if input == 100 {
        panic!("[mini-app] Crashing! =)");
    }
    format!("Hello, you wrote {}!", input)
}

use serde_json::{Value as JsonValue, Number};
use tauri::api::ipc::CallbackFn;
use tauri::hooks::InvokePayload;
use tauri::sealed::ManagerBase;
use tauri::test::MockRuntime;
use tauri::Wry;

fn main() {
    type R = MockRuntime;
    // type R = Wry;

    println!("[mini-app] Starting");
    let app = tauri::Builder::<R>::new()
        .invoke_handler(tauri::generate_handler![tauri_cmd_1, tauri_cmd_2])
        .build(tauri::generate_context!())
        .unwrap();
    println!("[mini-app] Tauri app is built");

    // app.run(|_, _| {});
    app.run(|app_handle, _event| {
        // Get the Tauri Window
        let windows = app_handle.manager().windows_lock();
        let main_window = windows.get("main").unwrap();

        // Create the message that will be sent to the backend
        let arg_name = String::from("input");

        // let value = JsonValue::String(String::from("aaa"));
        let value = JsonValue::Number(Number::from(3u32));
        let mut json_map = serde_json::map::Map::new();
        json_map.insert(arg_name, value);
        let payload = InvokePayload {
            // cmd: String::from("tauri_cmd_1"),
            cmd: String::from("tauri_cmd_2"),
            tauri_module: None,
            callback: CallbackFn(1248299581),
            error: CallbackFn(3880587747),
            inner: JsonValue::Object(json_map),
        };

        // Trigger a Tauri command by sending our crafted message
        let _ = main_window.clone().on_message(payload);

        // Exit after executing our command
        app_handle.exit(0)
    });
}
