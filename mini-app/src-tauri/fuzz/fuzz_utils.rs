use libafl_bolts::bolts_prelude::Cores;
use libafl_bolts::cli::FuzzerOptions;
use std::path::PathBuf;
use std::str::FromStr;

pub(crate) fn get_options(command: &str, libs_to_instrument: Vec<&str>) -> FuzzerOptions {
    FuzzerOptions {
        timeout: std::time::Duration::from_secs(5),
        verbose: true,
        stdout: String::from("/dev/stdout"),
        configuration: String::from("default configuration"),
        asan: false,
        asan_cores: Cores::from_cmdline("1-4").unwrap(),
        iterations: 0,
        harness: Some(PathBuf::from_str(command).unwrap()),
        harness_args: vec![],
        harness_function: String::from(""),
        libs_to_instrument: libs_to_instrument
            .into_iter()
            .map(|lib| lib.into())
            .collect(),
        cmplog: true,
        cmplog_cores: Cores::from_cmdline("1-4").unwrap(),
        detect_leaks: false,
        continue_on_error: false,
        allocation_backtraces: true,
        max_allocation: 1073741824,
        max_total_allocation: 4294967296,
        max_allocation_panics: true,
        disable_coverage: false,
        drcov: false,
        disable_excludes: true,  // check
        dont_instrument: vec![], // check
        tokens: vec![],          // check
        // input: vec![PathBuf::from_str("tauri_cmd_2_fuzz/corpus").unwrap()],
        input: vec![],
        output: PathBuf::from_str(&format!("{}_solutions", command)).unwrap(),
        // Doesn't work on MacOS
        // cores: Cores::from_cmdline("0").unwrap(),
        cores: Cores::from_cmdline("1-4").unwrap(),
        broker_port: 8888,
        remote_broker_addr: None,
        replay: None, // check
        repeat: None,
    }
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
/// fn create_app<R: tauri::Runtime>(mut builder: tauri::Builder<R>) -> tauri::App<R> {
///   builder
///     .invoke_handler(tauri::generate_handler![ping])
///     // remove the string argument on your app
///     .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
///     .expect("failed to build app")
/// }
///
/// use tauri::test::*;
/// let app = create_app(mock_builder());
/// let payload = create_invoke_payload("ping".into(), CommandArgs::new());
/// let res: Result<String, String> = invoke_command_and_stop(app, payload);
/// assert_eq!(res, Ok("pong".into()));
/// ```
pub fn invoke_command_and_stop<T: DeserializeOwned + Debug>(
    mut app: App<MockRuntime>,
    payload: InvokePayload,
) -> Result<T, T> {
    let w = app.get_window("main").expect("Could not get main window");

    let (tx, rx) = std::sync::mpsc::channel();

    let callback = payload.callback;
    let error = payload.error;
    let ipc = w.state::<Ipc>();
    ipc.0.lock().unwrap().insert(IpcKey { callback, error }, tx);
    w.on_message(payload).unwrap();

    app.run_iteration();
    let res: Result<JsonValue, JsonValue> =
        rx.recv().expect("Failed to receive result from command");
    res.map(|v| serde_json::from_value(v).unwrap())
        .map_err(|e| serde_json::from_value(e).unwrap())
}

use crate::RunEvent;
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
/// fn create_app<R: tauri::Runtime>(mut builder: tauri::Builder<R>) -> tauri::App<R> {
///   builder
///     .invoke_handler(tauri::generate_handler![ping])
///     // remove the string argument on your app
///     .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
///     .expect("failed to build app")
/// }
///
/// use tauri::test::*;
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

    let (tx, rx) = std::sync::mpsc::channel();

    let callback = payload.callback;
    let error = payload.error;
    let ipc = w.state::<Ipc>();
    ipc.0.lock().unwrap().insert(IpcKey { callback, error }, tx);
    w.on_message(payload).unwrap();
    app.run(event_callback);
}

/// Helper function to create a Tauri `InvokePayload`.
pub fn create_invoke_payload(cmd_name: String, command_args: CommandArgs) -> InvokePayload {
    let mut json_map = serde_json::map::Map::new();
    for (k, v) in command_args.inner {
        json_map.insert(k, v);
    }

    InvokePayload {
        cmd: cmd_name,
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
