/// Collection of helper functions that connects the fuzzer and Tauri
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use tauri::api::ipc::CallbackFn;
use tauri::test::MockRuntime;
use tauri::App;
use tauri::Builder;
use tauri::InvokePayload;
use tauri::Manager;

/// Minimal builder for a Tauri application using the `MockRuntime`
/// NOTE: if your Tauri command uses a state this won't work since it does manage any state
pub fn mock_builder_minimal() -> Builder<MockRuntime> {
    Builder::<MockRuntime>::new()
}

/// Invoke a command and get the Tauri command return value
pub fn invoke_command<T: DeserializeOwned + Debug>(
    app: App<MockRuntime>,
    payload: InvokePayload,
) -> Result<T, T> {
    let w = app.get_window("main").expect("Could not get main window");
    tauri::test::get_ipc_response::<T>(&w, payload)
}

/// Invoke a command but does not try to get the command return value
pub fn invoke_command_minimal(app: App<MockRuntime>, payload: InvokePayload) {
    let w = app.get_window("main").expect("Could not get main window");
    w.on_message(payload).unwrap();
}

/// Helper function to create a Tauri `InvokePayload`.
///
/// # Arguments
///
/// * `tauri_module` the module the invoked command is part of. Use `None` for a custom Tauri
/// command
/// * `cmd_name` name of the Tauri command invoked
/// * `command_args` arguments that are used for the Tauri command invocation
///
pub fn create_invoke_payload(
    tauri_module: Option<String>,
    cmd_name: &str,
    command_args: CommandArgs,
) -> InvokePayload {
    let mut json_command_args = serde_json::map::Map::new();
    for (k, v) in command_args.inner {
        json_command_args.insert(k, v);
    }
    match tauri_module {
        // The Tauri command invoked is a custom command
        None => {
            // The Tauri command invoked is a Tauri custom command and looks like this
            // InvokePayload {
            //     cmd: "<command name>",
            //     tauri_module: None,                 // module name
            //     callback: CallbackFn(0),
            //     error: CallbackFn(1),
            //     inner: Object {
            //         "path": String("..."),          // command parameter
            //         "options": Object {},           // command parameter
            //     },
            // }

            InvokePayload {
                cmd: cmd_name.into(),
                tauri_module: None,
                callback: CallbackFn(0),
                error: CallbackFn(1),
                inner: serde_json::Value::Object(json_command_args),
            }
        }

        Some(module) => {
            // The Tauri command invoked is a Tauri builtin command and looks like this
            // InvokePayload {
            //     cmd: "tauri",
            //     tauri_module: Some("Fs"),           // module name
            //     callback: CallbackFn(0),
            //     error: CallbackFn(1),
            //     inner: Object {
            //         "message": Object {
            //             "cmd": String("readFile"),  // command name
            //             "path": String("..."),      // command parameter
            //             "options": Object {},       // command parameter
            //         },
            //     },
            // }

            json_command_args.insert(
                String::from("cmd"),
                serde_json::Value::String(cmd_name.into()),
            );
            let mut inner_map = serde_json::map::Map::new();
            inner_map.insert(
                "message".into(),
                serde_json::Value::Object(json_command_args),
            );

            InvokePayload {
                cmd: "tauri".into(),
                tauri_module: Some(module),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                inner: serde_json::Value::Object(inner_map),
            }
        }
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
        value: impl Serialize,
    ) -> Option<serde_json::Value> {
        let key = key.into();
        self.inner.insert(
            key.clone(),
            serde_json::to_value(value).unwrap_or_else(|_| {
                panic!("Failed conversion to json value for parameter {}", key,)
            }),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    #[allow(dead_code)]
    #[tauri::command]
    fn test_command() -> String {
        String::from("foo")
    }

    #[test]
    fn test_invoke_command() {
        let app = mock_builder()
            .invoke_handler(tauri::generate_handler![test_command])
            .build(mock_context(noop_assets()))
            .unwrap();
        let payload = create_invoke_payload(None, "test_command", CommandArgs::new());
        let res = invoke_command::<String>(app, payload);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), "foo");
    }

    #[test]
    fn test_invoke_minimal() {
        let app = mock_builder_minimal()
            .invoke_handler(tauri::generate_handler![test_command])
            .build(mock_context(noop_assets()))
            .unwrap();
        let payload = create_invoke_payload(None, "test_command", CommandArgs::new());
        invoke_command_minimal(app, payload);
        // The goal is just to reach this point
        assert!(true);
    }
}
