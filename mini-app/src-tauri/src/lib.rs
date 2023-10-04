pub mod tauri_commands;
pub use tauri_commands::basic::*;
// pub use tauri_commands::shell::*;

use tauri::App as TauriApp;
use tauri::test::{mock_context, noop_assets, MockRuntime};

pub fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    tauri::Builder::<MockRuntime>::new()
        .invoke_handler(tauri::generate_handler![
            tauri_cmd_1,
            tauri_cmd_2,
        //     shell_command_0,
        //     shell_command_1,
        //     bin_sh,
        //     open_command
        ])
        .build(mock_context(noop_assets()))
}


pub fn bytes_input_to_u32(bytes_input: &[u8]) -> u32 {
    let mut array_input = [0u8; 4];
    for (dst, src) in array_input.iter_mut().zip(bytes_input) {
        *dst = *src
    }
    let res = u32::from_be_bytes(array_input);
    res
}
