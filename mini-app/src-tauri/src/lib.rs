pub mod commands;
pub use commands::*;
pub mod shell_injection;

use log::trace;
use serde_json::{Number, Value as JsonValue};
use std::collections::HashMap;

use tauri::api::ipc::CallbackFn;
use tauri::app::App as TauriApp;
use tauri::hooks::InvokePayload;
use tauri::sealed::ManagerBase;
use tauri::test::MockDispatcher;
use tauri::test::MockRuntime;
use tauri::test::{mock_context, noop_assets};
use tauri::RunEvent;

pub fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    tauri::Builder::<MockRuntime>::new()
        .invoke_handler(tauri::generate_handler![tauri_cmd_1, tauri_cmd_2])
        .build(mock_context(noop_assets()))
}

pub fn call_tauri_cmd(app: TauriApp<MockRuntime>, payload: InvokePayload) {
    app.run(move |app_handle, event| {
        trace!("[call_tauri_cmd] Received runtime event: {:?}", event);
        match event {
            // We have received a message that all windows were closed
            RunEvent::ExitRequested { .. } => {}
            RunEvent::Exit => {}
            _ => {
                // Get the Tauri Window
                let windows = app_handle.manager().windows_lock();
                let main_window = windows.get("main").unwrap();

                // Trigger a Tauri command by sending our crafted message
                let _ = main_window.clone().on_message(payload.clone());

                // Send a message to close the window
                let _ = <MockDispatcher as tauri_runtime::Dispatch<()>>::close(
                    &main_window.clone().window.dispatcher,
                );
            }
        }
    });
    trace!("[call_tauri_cmd] Finished calling command");
}

fn payload_for_tauri_cmd(
    cmd_name: String,
    command_args: HashMap<String, JsonValue>,
) -> InvokePayload {
    let mut json_map = serde_json::map::Map::new();
    for (k, v) in command_args {
        json_map.insert(k, v);
    }

    InvokePayload {
        cmd: cmd_name,
        tauri_module: None,
        callback: CallbackFn(0),
        error: CallbackFn(1),
        inner: JsonValue::Object(json_map),
    }
}

#[allow(dead_code)]
pub fn payload_for_tauri_cmd_1(bytes: &[u8]) -> InvokePayload {
    let input = String::from_utf8_lossy(bytes).to_string();
    let arg_name = String::from("input");
    let arg_value = JsonValue::String(input);
    let mut args = HashMap::new();
    args.insert(arg_name, arg_value);
    payload_for_tauri_cmd(String::from("tauri_cmd_1"), args)
}

#[allow(dead_code)]
pub fn payload_for_tauri_cmd_2(bytes: &[u8]) -> InvokePayload {
    let input = bytes_input_to_u32(bytes);
    let arg_name = String::from("input");
    let arg_value = JsonValue::Number(Number::from(input));
    let mut args = HashMap::new();
    args.insert(arg_name, arg_value);
    payload_for_tauri_cmd(String::from("tauri_cmd_2"), args)
}

pub fn bytes_input_to_u32(bytes_input: &[u8]) -> u32 {
    let mut array_input = [0u8; 4];
    for (dst, src) in array_input.iter_mut().zip(bytes_input) {
        *dst = *src
    }
    let res = u32::from_be_bytes(array_input);
    res
}
