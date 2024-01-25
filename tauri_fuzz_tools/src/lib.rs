mod fuzz_utils;
pub use fuzz_utils::*;
pub mod policies;

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    #[tauri::command]
    fn test_command() {
        println!("testing command");
    }

    #[test]
    fn test_invoke_command_minimal() {
        let app = mock_builder_minimal()
            .invoke_handler(tauri::generate_handler![test_command])
            .build(mock_context(noop_assets()))
            .unwrap();
        let payload = create_invoke_payload("test_command", CommandArgs::new());
        invoke_command_minimal(app, payload);
        assert!(true);
    }

    #[test]
    fn test_invoke_command_and_stop() {
        let app = mock_builder()
            .invoke_handler(tauri::generate_handler![test_command])
            .build(mock_context(noop_assets()))
            .unwrap();
        let payload = create_invoke_payload("test_command", CommandArgs::new());
        let res = invoke_command_and_stop::<()>(app, payload);
        assert!(res.is_ok());
    }
}
