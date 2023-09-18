pub mod tauri_commands;
pub use tauri_commands::basic::*;
pub use tauri_commands::shell::*;

use log::trace;
use std::collections::HashMap;

use serde_json::Value as JsonValue;
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
        .invoke_handler(tauri::generate_handler![
            tauri_cmd_1,
            tauri_cmd_2,
            shell_command_0,
            shell_command_1,
            bin_sh,
            open_command
        ])
        .build(mock_context(noop_assets()))
}

pub fn call_tauri_cmd(app: TauriApp<MockRuntime>, payload: InvokePayload) {
    // ) -> Result<TauriApp<MockRuntime>, tauri::Error> {
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
                trace!("[call_tauri_cmd] Calling with {:#?}", payload);
                main_window
                    .clone()
                    .on_message(payload.clone())
                    .expect(&format!(
                        "Failed to run tauri commands with payload: {:?}",
                        payload
                    ));

                // Send a message to close the window
                let _ = <MockDispatcher as tauri_runtime::Dispatch<()>>::close(
                    &main_window.clone().window.dispatcher,
                );
            }
        }
    });
    trace!("[call_tauri_cmd] Finished calling command");
}

pub fn payload_for_tauri_cmd(
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

pub fn bytes_input_to_u32(bytes_input: &[u8]) -> u32 {
    let mut array_input = [0u8; 4];
    for (dst, src) in array_input.iter_mut().zip(bytes_input) {
        *dst = *src
    }
    let res = u32::from_be_bytes(array_input);
    res
}
