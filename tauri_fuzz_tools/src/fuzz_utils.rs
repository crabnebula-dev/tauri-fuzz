use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fmt::Debug;
use tauri::api::ipc::CallbackFn;
use tauri::test::MockRuntime;
use tauri::test::{Ipc, IpcKey};
use tauri::App;
use tauri::AppHandle;
use tauri::Builder;
use tauri::InvokePayload;
use tauri::Manager;
use tauri::RunEvent;

/// Minimal builder for a Tauri application using the `MockRuntime`
pub fn mock_builder_minimal() -> Builder<MockRuntime> {
    Builder::<MockRuntime>::new()
}

/// Invoke a Tauri command given an `InvokePayload`.
///
/// The application processes the command and stops.
///
/// # Examples
///
/// ```rust
///
/// #[tauri::command]
/// fn ping() -> &'static str {
///   "pong"
/// }
///
/// use tauri_fuzz_tools::*;
/// use tauri::test::*;
/// fn create_app<R: tauri::Runtime>(mut builder: tauri::Builder<R>) -> tauri::App<R> {
///   builder
///     .invoke_handler(tauri::generate_handler![ping])
///     // remove the string argument on your app
///     .build(mock_context(noop_assets()))
///     .expect("failed to build app")
/// }
///
/// let app = create_app(mock_builder());
/// let payload = create_invoke_payload("ping".into(), CommandArgs::new());
/// let res: Result<String, String> = invoke_command_and_stop(app, payload);
/// assert_eq!(res, Ok("pong".into()));
/// ```
pub fn invoke_command_and_stop<T: DeserializeOwned + Debug>(
    mut app: App<MockRuntime>,
    payload: InvokePayload,
) -> Result<T, T> {
    let (tx, rx) = std::sync::mpsc::channel();
    let w = app.get_window("main").expect("Could not get main window");
    let callback = payload.callback;
    let error = payload.error;

    let ipc = w.state::<Ipc>();
    ipc.insert_ipc(IpcKey { callback, error }, tx);
    w.on_message(payload).unwrap();

    app.run_iteration();
    let res: Result<JsonValue, JsonValue> =
        rx.recv().expect("Failed to receive result from command");
    res.map(|v| serde_json::from_value(v).unwrap())
        .map_err(|e| serde_json::from_value(e).unwrap())
}

/// Invoke a Tauri command given an `InvokePayload`.
///
/// A callback can be provided to interact with future events.
///
/// # Examples
///
/// ```rust
/// #[tauri::command]
/// fn ping() -> &'static str {
///   "pong"
/// }
///
/// use tauri_fuzz_tools::*;
/// use tauri::test::*;
/// fn create_app<R: tauri::Runtime>(mut builder: tauri::Builder<R>) -> tauri::App<R> {
///   builder
///     .invoke_handler(tauri::generate_handler![ping])
///     // remove the string argument on your app
///     .build(mock_context(noop_assets()))
///     .expect("failed to build app")
/// }
///
/// let app = create_app(mock_builder());
/// let payload = create_invoke_payload("ping".into(), CommandArgs::new());
/// // Exit the application after processing the command
/// invoke_command(app, payload, move |app_handle, event| {
///   match event {
///      tauri::RunEvent::Ready => app_handle.exit(0),
///      _ => (),
///   }
/// });
/// ```
pub fn invoke_command<F: FnMut(&AppHandle<MockRuntime>, RunEvent) + 'static>(
    app: App<MockRuntime>,
    payload: InvokePayload,
    event_callback: F,
) {
    let w = app.get_window("main").expect("Could not get main window");

    let (tx, _rx) = std::sync::mpsc::channel();
    let callback = payload.callback;
    let error = payload.error;
    let ipc = w.state::<Ipc>();
    ipc.insert_ipc(IpcKey { callback, error }, tx);
    w.on_message(payload).unwrap();
    app.run(event_callback);
}

/// Invoke a command but does not try to get the command return value
pub fn invoke_command_minimal(app: App<MockRuntime>, payload: InvokePayload) {
    let w = app.get_window("main").expect("Could not get main window");
    w.on_message(payload).unwrap();
}

/// Helper function to create a Tauri `InvokePayload`.
pub fn create_invoke_payload(cmd_name: &str, command_args: CommandArgs) -> InvokePayload {
    let mut json_map = serde_json::map::Map::new();
    for (k, v) in command_args.inner {
        json_map.insert(k, v);
    }

    InvokePayload {
        cmd: cmd_name.into(),
        tauri_module: None,
        callback: CallbackFn(0),
        error: CallbackFn(1),
        inner: serde_json::Value::Object(json_map),
    }
}

/// A wrapper around HashMap to facilitate `InvokePayload` creation.
#[derive(Default)]
pub struct CommandArgs {
    /// Inner type
    pub inner: HashMap<String, serde_json::Value>,
}

impl CommandArgs {
    /// Create a `CommandArgs`.
    pub fn new() -> CommandArgs {
        CommandArgs {
            inner: HashMap::new(),
        }
    }

    /// Insert a key, value pair that will be converted into the correct json type.
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Option<serde_json::Value> {
        self.inner.insert(key.into(), value.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    #[allow(dead_code)]
    #[tauri::command]
    fn test_command() {}

    // #[test]
    #[allow(dead_code)]
    fn invoke_1_command() {
        let app = mock_builder()
            .invoke_handler(tauri::generate_handler![test_command])
            .build(mock_context(noop_assets()))
            .unwrap();
        let payload = create_invoke_payload("test_command", CommandArgs::new());
        let res = invoke_command_and_stop::<String>(app, payload);
        assert!(res.is_ok());
    }
}
