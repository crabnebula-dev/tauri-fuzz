/// Collection of helper functions that connects the fuzzer and Tauri
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{MockRuntime, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::Builder;
use tauri::WebviewWindow;
use tauri_utils::acl::capability::CapabilityFile::{self, *};

/// Minimal builder for a Tauri application using the `MockRuntime`
/// NOTE: if your Tauri command uses a state this won't work since it does manage any state
pub fn mock_builder_minimal() -> Builder<MockRuntime> {
    Builder::<MockRuntime>::new()
}

/// Invoke a command and get the Tauri command return value
pub fn invoke_command<T: DeserializeOwned + Debug, E: DeserializeOwned + Debug>(
    webview: &WebviewWindow<MockRuntime>,
    request: InvokeRequest,
) -> Result<T, E> {
    let res = tauri::test::get_ipc_response(&webview, request);
    res.map(|response| {
        let deserialized_res = response.deserialize();
        deserialized_res.expect("Error while deserializing the command response")
    })
    .map_err(|err| {
        serde_json::from_value(err).expect("Error while deserializing the error response")
    })
}

/// Invoke a command but does not try to get the command return value
pub fn invoke_command_minimal(webview: WebviewWindow<MockRuntime>, request: InvokeRequest) {
    webview.on_message(
        request,
        Box::new(move |_window, _cmd, _response, _callback, _error| ()),
    )
}

/// Url used by tauri commands
#[cfg(not(any(windows, target_os = "android")))]
const TAURI_PROTOCOL_URL: &str = "tauri://localhost";
#[cfg(any(windows, target_os = "android"))]
const TAURI_PROTOCOL_URL: &str = "http://tauri.localhost";

/// Helper function to create a Tauri `InvokeRequest`.
///
/// # Arguments
///
/// * `tauri_module` the module the invoked command is part of. Use `None` for a custom Tauri command
/// * `cmd_name` name of the Tauri command invoked
/// * `command_args` arguments that are used for the Tauri command invocation
///
pub fn create_invoke_request(
    tauri_plugin: Option<String>,
    cmd_name: &str,
    command_args: CommandArgs,
) -> InvokeRequest {
    let mut json_command_args = serde_json::map::Map::new();
    for (k, v) in command_args.inner {
        json_command_args.insert(k, v);
    }

    match tauri_plugin {
        // The Tauri command invoked is a custom command

        // #### Template for a custom InvokeRequest
        // InvokeRequest {
        //     cmd: "greet",
        //     callback: CallbackFn(
        //         611932980,
        //     ),
        //     error: CallbackFn(
        //         1842704042,
        //     ),
        //     url: Url {
        //         scheme: "http",
        //         cannot_be_a_base: false,
        //         username: "",
        //         password: None,
        //         host: Some(
        //             Ipv4(
        //                 127.0.0.1,
        //             ),
        //         ),
        //         port: Some(
        //             1430,
        //         ),
        //         path: "/",
        //         query: None,
        //         fragment: None,
        //     },
        //     body: Json(
        //         Object {
        //             "name": String(""),
        //         },
        //     ),
        //     headers: {},
        // }
        None => InvokeRequest {
            cmd: cmd_name.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: TAURI_PROTOCOL_URL.parse().unwrap(),
            body: InvokeBody::from(serde_json::value::Value::Object(json_command_args)),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },

        Some(plugin) => {
            // #### Template for a plugin InvokeRequest
            // InvokeRequest {
            //     cmd: "plugin:fs|read_file",
            //     callback: CallbackFn(
            //         3255320200,
            //     ),
            //     error: CallbackFn(
            //         3097067861,
            //     ),
            //     url: Url {
            //         scheme: "http",
            //         cannot_be_a_base: false,
            //         username: "",
            //         password: None,
            //         host: Some(
            //             Ipv4(
            //                 127.0.0.1,
            //             ),
            //         ),
            //         port: Some(
            //             1430,
            //         ),
            //         path: "/",
            //         query: None,
            //         fragment: None,
            //     },
            //     body: Json(
            //         Object {
            //             "options": Object {},
            //             "path": String("README.md"),
            //         },
            //     ),
            //     headers: {},
            // }

            // Command name has pattern
            // "plugin:{plugin_name}|{command_name}"
            let mut cmd = String::from("plugin:");
            cmd.push_str(&plugin);
            cmd.push('|');
            cmd.push_str(cmd_name);

            InvokeRequest {
                cmd,
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: TAURI_PROTOCOL_URL.parse().unwrap(),
                body: InvokeBody::from(serde_json::value::Value::Object(json_command_args)),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            }
        }
    }
}

/// Setup a Tauri application context which can call commands from a Tauri plugin.
/// This involves passing the plugin permission and defining the application capability.
pub fn setup_context_with_plugin<R: tauri::Runtime>(
    context: &mut tauri::Context<R>,
    plugin_name: &str,
    permission_file_toml: &str,
    capability: &str,
) {
    let runtime_authority = context.runtime_authority_mut();

    // The acl of our application contains the `read-files` permission from the fs plugin
    let permission_file: tauri_utils::acl::manifest::PermissionFile =
        toml::from_str(permission_file_toml).unwrap();
    let manifest = tauri_utils::acl::manifest::Manifest::new(vec![permission_file], None);
    let mut acl = std::collections::BTreeMap::new();
    acl.insert(plugin_name.to_string(), manifest);

    // Capability of our mock app declare the use of the `fs:read-files` permission
    let capability_file = CapabilityFile::from_str(capability).unwrap();
    let Capability(capability) = capability_file else {
        unreachable!()
    };
    let mut capability_map = std::collections::BTreeMap::new();
    capability_map.insert(capability.identifier.clone(), capability.clone());

    // Resolved capabilities against the acl
    let resolved = tauri_utils::acl::resolved::Resolved::resolve(
        &acl,
        capability_map,
        tauri_utils::platform::Target::current(),
    )
    .unwrap();

    // Setup our custom `RuntimeAuthority` in our application context
    *runtime_authority = tauri::ipc::RuntimeAuthority::new(acl, resolved);
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
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let request = create_invoke_request(None, "test_command", CommandArgs::new());
        let res: Result<String, String> = invoke_command(&webview, request);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), "foo");
    }

    fn path_to_foo() -> std::path::PathBuf {
        let mut path = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        path.push("assets");
        path.push("foo.txt");
        path
    }

    use tauri_plugin_fs::FsExt;
    #[test]
    fn test_invoke_command_plugin() {
        // Trimmed `read-files` permission from the Fs plugin
        const FS_READ_FILE_PERMISSION: &str = r#"
[[permission]]
identifier = "read-files"
description = "This enables file read related commands without any pre-configured accessible paths."
commands.allow = [
    "read_file",
]"#;

        // Capability given to our mock app, use `fs:read-files` permission
        const CAPABILITY: &str = r#"{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "fs:read-files"
  ]
}"#;

        let mut context = mock_context(noop_assets());
        setup_context_with_plugin(&mut context, "fs", FS_READ_FILE_PERMISSION, CAPABILITY);

        // Build the app with our custom context and init the plugin
        let app = mock_builder()
            .plugin(tauri_plugin_fs::init())
            .invoke_handler(tauri::generate_handler![])
            .build(context)
            .unwrap();

        // Modify the scope of the fs plugin
        let scope = app.fs_scope();
        scope.allow_file(path_to_foo().to_str().unwrap());

        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let mut args = CommandArgs::new();
        args.insert("path", path_to_foo().to_string_lossy().into_owned());

        let request = create_invoke_request(Some("fs".into()), "read_file", args);

        let res: Result<Vec<u8>, String> = invoke_command(&webview, request);
        assert!(res.is_ok());
        let res = unsafe { String::from_utf8_unchecked(res.unwrap()) };

        // Remove trailing linebreaks for every platform
        let stripped_res = res
            .strip_suffix("\r\n")
            .or(res.strip_suffix('\n'))
            .unwrap_or(&res);

        assert_eq!(stripped_res, "foo");
    }

    #[test]
    fn test_invoke_minimal() {
        let app = mock_builder_minimal()
            .invoke_handler(tauri::generate_handler![test_command])
            .build(mock_context(noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let request = create_invoke_request(None, "test_command", CommandArgs::new());
        invoke_command_minimal(webview, request);
        // The goal is just to reach this point
    }
}
