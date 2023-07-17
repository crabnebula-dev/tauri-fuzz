pub mod commands;
pub use commands::*;

// use tauri::app::App as TauriApp;
// use tauri::test::MockRuntime;
//
// #[no_mangle]
// // No mangle is used for fuzzing when using Qemu
// pub fn setup_tauri_mock() -> TauriApp<MockRuntime> {
//     tauri::Builder::<MockRuntime>::new()
//         .invoke_handler(tauri::generate_handler![tauri_cmd_1, tauri_cmd_2])
//         .build(tauri::generate_context!())
//         .unwrap()
// }
