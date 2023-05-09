# Finding Proper Fuzzer Targets

The Tauri Endpoints, Tauri Commands, Tauri API were discussed.
For effective fuzzing it is best to have as few dependencies or inter
connected components as possible.

> If code isn't well unit-tested, developing fuzzers is harder but still extremely beneficial. Writing fuzzers can make code easier to unit test:
> It may prompt refactoring code to expose an API that's easier to fuzz, such as turning a binary into a thin wrapper around a library. These changes also make the code easier to unit test.
> A coverage-guided fuzzer will produce a corpus, or set of "interesting" inputs. The corpus often includes edge cases and other unanticipated inputs that are useful when adding unit tests.
[^unit-tests]

[^unit-tests]: [https://fuchsia.dev/fuchsia-src/contribute/testing/fuzz_testing#unit-tests](https://fuchsia.dev/fuchsia-src/contribute/testing/fuzz_testing#unit-tests)

Current state is interconnected due to:

- Feature Flags
- OS depending APIs
- Components depending on underlying features
- Complex structs that contain many fields (at least 20 sometimes) reliant on all mix of the above

## API

- Very thin layer around rust standard library, some system components (native dialogs) and very little logic implemented
- Simple to write a fuzzer for but hard to effectively gather feedback and interesting events, as most logic happens
inside the rust standard library or the system component and not in the code logic of the API function
- Logic that does happen inside API functions is typically limited to file system items, as they have their own "root" directory
- Does require only very few mocked code
- Exposed as `pub` and accessible for fuzzing

## Endpoints

- Complexity with feature flags, scopes and additional interaction with the system components
- Not exposed as `pub` and therefore **inaccessible for fuzzing**
- Possibly good candidate for figuring out the mock runtime
- requires some mocked code
- requires fork or refactor of Tauri

## Commands

- Most complex and upper level layer, which gets invoked via the `tauri-runtime`
- The `tauri-runtime` is handling the IPC with the webview and passes already deserialized
data types to the commands
- Can be implemented by application developers and are the main target from an auditor perspective
- Creating simple fuzzer boilerplate code for these would provide the most benefit for app devs and auditors
- most likely requires full mock runtime
- Exposed as `pub` and accessible for fuzzing/handled by a macro and everything marked with `#[command]`
- most likely requires virtual runtime (qemu mode)

```rust
use tauri_macros::{command, generate_handler};

#[command]
fn command_one() {
  println!("command one called");
  // Some advanced logic here
}
```

Each custom implemented command has some logic. The Tauri commands interact with system components
and custom commands are similar but can contain any kind of logic.
Writing a *generic command boilerplate fuzzer* is the goal but tweaks for the logic are most likely
needed for most cases.

### How a command is called

1. `[frontend]` call @tauri-apps/api `invoke` with command `invoke("my_command", {data: "payload"})`
2. `[frontend]` `invoke` calls the underlying ipc handler set by `wry`
3. `[wry]` sends received message to the ipc handler set in the `tauri-runtime`
4. `[tauri]` `Manager` receives message from the runtime
5. `[tauri]` `Manager` sees if passed command exists, and if so, calls it
6. `[tauri]` `my_command` received deserialized data, runs, and returns
7. `[tauri]` `Manager` serializes the return value and sends it to the runtime to be eval'd
8. `[tauri-runtime]` sends the eval script to wry
9. `[wry]` injects the eval script
10. `[frontend]` the callback created in #1 is called with the serialized return value

### Harness

Because of the following reasons, tooling around fuzzing Tauri commands is based around tool-assisted fuzzing and not automatically generated fuzzing.

Tauri commands can be arbitrary and the harness needs to be decided based on what the
command is calling internally.

Some questions to ask:

- Is it using non-`Deserialize` items (`Window`, `AppHandle`, `State`)?
- Is it accessign the Tauri state?
- Is it modifying the Tauri state?
- Is it calling other internal Tauri APIs?
- What is the actual code logic?

Example code calling native functions from the wild[^source-command-native]:

```rust
#[tauri::command]
fn start_tracking(state: State<'_, BgInput>) -> Result<(), String> {
    {
        let current_hook_id = state.listen_hook_id.read().unwrap();
        if current_hook_id.is_some() {
            return Err("Already active".into());
        }
    }
    
    let tx = state.tx.clone();
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(move |cmd| {
            let rpc: String = match cmd {
                KeyCommand::Escape => "cmd:cancel".to_string(),
                KeyCommand::Return => "cmd:submit".to_string(),
                KeyCommand::Delete | KeyCommand::BackSpace => "cmd:delete".to_string(),
                KeyCommand::Key(key) => format!("key:{}", key),
            };
            tx.send(rpc).unwrap();
        }));
    }
    let Ok(hook) = (unsafe {SetWindowsHookExA(WH_KEYBOARD_LL, Some(raw_callback), None, 0)}) else {
        return Err("Could not start listener".into());
    };
    let mut wr = state.listen_hook_id.write().unwrap();
    *wr = Some(hook);
    Ok(())
}
```

Example code calling Tauri functions from the wild[^source-command-tauri]:

```rust
#[tauri::command]
fn app_close(app_handle: tauri::AppHandle) {
    let Some(window) = app_handle.get_window("main") else {
        return app_handle.exit(0);
    };
    app_handle.save_window_state(StateFlags::all()).ok(); // don't really care if it saves it

    if let Err(_) = window.close() {
        return app_handle.exit(0);
    }
}
```

To effectivly fuzz both methods a mock runtime is needed, where behavior should be changed based on the command expectation.
Additionally, some native system calls need to be mocked if the libraries or systems are not in scope.

[^source-command-native]: [https://github.com/mmpneo/curses/](https://github.com/mmpneo/curses/blob/db372290984ab9d1367f862af041a1f6441f4006/src-tauri/src/services/keyboard/mod.rs#LL122C1-L150C1)

[^source-command-tauri]: [https://github.com/mmpneo/curses/](https://github.com/mmpneo/curses/blob/db372290984ab9d1367f862af041a1f6441f4006/src-tauri/src/main.rs#LL42C1-L52C2)
