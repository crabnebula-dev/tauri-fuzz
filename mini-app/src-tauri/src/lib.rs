use serde_json::{Number, Value as JsonValue};
use tauri::api::ipc::CallbackFn;
use tauri::app::App as TauriApp;
use tauri::hooks::InvokePayload;
use tauri::sealed::ManagerBase;
use tauri::test::MockDispatcher;
use tauri::test::MockRuntime;
use tauri::RunEvent;

#[no_mangle]
pub fn setup_tauri_mock() -> TauriApp<MockRuntime> {
    tauri::Builder::<MockRuntime>::new()
        .invoke_handler(tauri::generate_handler![tauri_cmd_1, tauri_cmd_2])
        .build(tauri::generate_context!())
        .unwrap()
}

mod commands;
use commands::*;

// #[no_mangle]
// pub fn call_tauri_cmd_1(app: TauriApp<MockRuntime>) {
//     app.run(|app_handle, _event| {
//         // Get the Tauri Window
//         let windows = app_handle.manager().windows_lock();
//         let main_window = windows.get("main").unwrap();
//
//         // Create the message that will be sent to the backend
//         let arg_name = String::from("input");
//
//         // Payload for tauri_cmd_1
//         let value = JsonValue::String(String::from("aaa"));
//         let mut json_map = serde_json::map::Map::new();
//         json_map.insert(arg_name.clone(), value);
//         let payload1 = InvokePayload {
//             cmd: String::from("tauri_cmd_1"),
//             tauri_module: None,
//             callback: CallbackFn(0),
//             error: CallbackFn(1),
//             inner: JsonValue::Object(json_map),
//         };
//
//         // Trigger a Tauri command by sending our crafted message
//         let _ = main_window.clone().on_message(payload1);
//
//         // Exit after executing our command
//         // app_handle.exit(0)
//     });
//     println!("toto");
// }

#[no_mangle]
pub fn call_tauri_cmd_2(app: TauriApp<MockRuntime>, input: u32) {
    app.run(move |app_handle, event| {
        println!("[mini-app::lib] Received runtime event: {:?}", event);
        match event {
            // We have received a message that all windows were closed
            RunEvent::ExitRequested { .. } => {}
            RunEvent::Exit => {}
            _ => {
                // Get the Tauri Window
                let windows = app_handle.manager().windows_lock();
                let main_window = windows.get("main").unwrap();

                // Create the message that will be sent to the backend
                let arg_name = String::from("input");

                // Payload for tauri_cmd_2
                let value = JsonValue::Number(Number::from(input));
                let mut json_map = serde_json::map::Map::new();
                json_map.insert(arg_name.clone(), value);
                let payload2 = InvokePayload {
                    cmd: String::from("tauri_cmd_2"),
                    tauri_module: None,
                    callback: CallbackFn(0),
                    error: CallbackFn(1),
                    inner: JsonValue::Object(json_map),
                };

                // Trigger a Tauri command by sending our crafted message
                let _ = main_window.clone().on_message(payload2);

                // Send a message to close the window
                let _ = <MockDispatcher as tauri_runtime::Dispatch<()>>::close(
                    &main_window.clone().window.dispatcher,
                );
            }
        }
    });
    println!("[mini_app] Finished calling command");
}
