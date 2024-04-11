# Integrate Fuzzing in Your Tauri App

This guide will showcase how you can add fuzzing to an existing Tauri application.
At the time of writing we are targeting Tauri `v1.x` but the steps are compatible with `v2.x`
applications.

In the concrete example we will show how to add fuzzing to the Tauri application `mini-app`, which
is part of the fuzzer [repository](https://github.com/crabnebula-dev/tauri-fuzzer/).

We will implement to fuzz the Tauri command `mini-app::tauri_commands::file_access::read_foo_file`.

## Prepare Commands to Fuzz

### Convert Project to a Crate

> **Note**: This is only needed for Tauri `v1.x`. Starting from Tauri `v2.x` this will be part of the default structure,
as mobile applications need to be a library.
> On `v2.x` it is only important that the commands are `pub` in the `lib.rs`.

By default a Tauri application is just a binary.
We want the Rust code to also be a crate (library) so that the fuzzer is able to call the application
Tauri commands.

To achieve this, we need to create a `lib.rs`.

In our concrete example case we need to add a `lib.rs` file in `mini-app/src-tauri/src`:

```bash
touch mini-app/src-tauri/src/lib.rs
```

### Expose Commands in `lib.rs`

For the fuzzer to be able to find the commands reliable, they need to be publicly exposed (`pub`)
in the `lib.rs`.
The commands don't need to be written in the `lib.rs` file and it is only needed to reference
them.

To expose the `file_access::read_foo_file` command we need to modify the `lib.rs`:

`mini-app/src-tauri/src/lib.rs`:
 ```rust,ignore
 pub mod tauri_commands;
 pub use tauri_commands::file_access::read_foo_file
 ```

The application layout should look similar to this:

`mini-app/src-tauri/src/main.rs`:
```rust,ignore
use mini_app::*;
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri_fuzz_tools::{create_invoke_payload, invoke_command, CommandArgs};

fn main() {
  ...
    let context = tauri::generate_context!();
   
    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            mini_app::file_access::read_foo_file
        ])
        .build(mock_context(noop_assets()))
        .expect("Failed to init Tauri app");
   ...
}
```

`mini-app/src-tauri/src/tauri_commands/file_access.rs`:
```rust,ignore
#[tauri::command]
/// Read the file `assets/foo.txt`
pub fn read_foo_file() -> String {
    trace!("[read_foo_file] Entering");
    let path = get_foo_path();
    let mut content = String::new();
    let mut file = File::open(path).unwrap();
    file.read_to_string(&mut content).unwrap();
    content
}
```



## Setup the `fuzz` Directory 

Create a `fuzz` directory inside your `src-tauri` folder and copy
the directory `mini-app-fuzz` into the `src-tauri/fuzz` directory of your application.

```bash
mkdir -p mini-app/src-tauri/fuzz
cp -r mini-app-fuzz/* mini-app/src-tauri/fuzz
cd mini-app/src-tauri/fuzz
```

### Configure Dependencies

Add your Tauri app as a dependency in the `mini-app/src-tauri/fuzz/mini-app-fuzz/Cargo.toml` file.
It should look like this:

```toml
[dependencies]
// Your Tauri app
mini-app = { path = "../" }
```
When using the fuzzer inside of this fuzzer repository workspace the following dependencies can be configured
to use the workspace:

```toml
tauri = { workspace = true, features = ["api-all"] }
tauri-utils = { workspace = true }
tauri_fuzz_tools = { workspace = true }
libafl = { workspace = true }
libafl_bolts = { workspace = true }
libafl_frida = { workspace = true }
libafl_targets = { workspace = true }
frida-gum = { workspace = true }
```

Otherwise please check out the example configuration below.

#### Example `Cargo.toml`

```toml
[package]
# Your package name
name = "mini-app-fuzz"
version = "0.0.0"
publish = false
edition = "2021"

[dependencies]
# Your Tauri app
mini-app = { path = "../" }

# Logging
log = "0.4"
env_logger = "*"
color-backtrace = "0.5"

# Frida binary analyser
frida-gum = { version = "0.13.2", features = [
  "auto-download",
  "event-sink",
  "invocation-listener",
] }

# Our fork of LibAFL
libafl = { git = "ssh://git@github.com/crabnebula-dev/LibAFL.git", features = [
  "std",
  "llmp_compression",
  "llmp_bind_public",
  "frida_cli",
], branch = "tauri" } #,  "llmp_small_maps", "llmp_debug"]}
libafl_bolts = { git = "ssh://git@github.com/crabnebula-dev/LibAFL.git", branch = "tauri" }
libafl_frida = { git = "ssh://git@github.com/crabnebula-dev/LibAFL.git", features = [
  "cmplog",
], branch = "tauri" }
libafl_targets = { git = "ssh://git@github.com/crabnebula-dev/LibAFL.git", features = [
  "sancov_cmplog",
], branch = "tauri" }

# Official Tauri
tauri = { version = "1.5", default-features = false, features = [
  "test",
  "tracing",
] }
tauri-utils = "1.5"

# Utility crate to connect the fuzzer and Tauri
tauri_fuzz_tools = { git = "ssh://git@github.com/crabnebula-dev/tauri-fuzzer.git", branch = "main" }

# Fuzzer and policy used
[lib]
name = "fuzzer"
path = "fuzzer/lib.rs"

# Your fuzz target
[[bin]]
name = "fuzz_read_foo_file"
path = "fuzz_targets/fuzz_read_foo_file.rs"
doc = false
```

## Write a Fuzz Target

This step creates a fuzz target, based on an template file.
The fuzz target will be compiled as a binary, using the exposed
commands.


Following our example process you need to:

1. Make a copy of the template from `mini-app-fuzz/fuzz_targets/template.rs`
    ```bash
    cp -f fuzz_targets/template.rs fuzz_targets/fuzz_read_foo_file.rs
    ```
2. Fill the corresponding information marked with `#CUSTOMIZE` in the comments.
    - command name is `read_foo_file`
        ```rust,ignore
        const COMMAND_NAME: &str = "read_foo_file";
        ```
    - in the harness, generate the Tauri app with the handle `mini-app::tauri_commands::file_access::read_foo_file`
        ```rust,ignore
        .invoke_handler(tauri::generate_handler![mini_app::tauri_commands::file_access::read_foo_file])
        ```
    - fill the `create_payload` to invoke your Tauri command with the right parameters
        ```rust,ignore
        fn create_payload(_bytes: &[u8]) -> InvokePayload {
            let args = CommandArgs::new();
            create_invoke_payload(None, COMMAND_NAME, args)
        }
        ```
    - specify the `FuzzPolicy` you want to apply
      ```rust,ignore
      fuzzer::policies::file_policy::no_access_to_filenames()
      ```


#### Example Target

This example target code is taken from `mini-app-fuzz/fuzz_targets/read_foo_file.rs`:

```rust,ignore
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;
use tauri_fuzz_tools::{
    create_invoke_payload, invoke_command_minimal, mock_builder_minimal, CommandArgs,
};

const COMMAND_NAME: &str = "read_foo_file";

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder_minimal()
        .invoke_handler(tauri::generate_handler![
            mini_app::tauri_commands::file_access::read_foo_file
        ])
        .build(mock_context(noop_assets()))
}

pub fn main() {
    let addr = mini_app::tauri_commands::file_access::read_foo_file as *const () as usize;
    let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _res = invoke_command_minimal(app, create_payload(input.bytes()));
        ExitKind::Ok
    };

    fuzzer::main(
        harness,
        options,
        addr,
        fuzzer::policies::file_policy::no_access_to_filenames(),
    );
}

fn create_payload(_bytes: &[u8]) -> InvokePayload {
    let args = CommandArgs::new();
    create_invoke_payload(None, COMMAND_NAME, args)
}
```

With the function `mini-app::tauri_commands::file_access::read_foo_file`:

```rust,ignore
#[tauri::command]
/// Read the file `assets/foo.txt`
pub fn read_foo_file() -> String {
    trace!("[read_foo_file] Entering");
    let path = get_foo_path();
    let mut content = String::new();
    let mut file = File::open(path).unwrap();
    file.read_to_string(&mut content).unwrap();
    content
}
```



### Add your fuzz target as a binary

In your `Cargo.toml` file add the fuzz target you created above as a binary.


```toml
[[bin]]
name = "fuzz_read_foo_file"
path = "fuzz_targets/fuzz_read_foo_file.rs"
doc = false
```

### Optional: change the fuzzer options

You can change the fuzzing options in `fuzzer/fuzzer_options.rs`.
For example the number of core used for fuzzing.

### Optional: create a new `FuzzPolicy`

You can create a `FuzzPolicy` in `fuzzer/policies/`.
More information in the following sections.


## Start Fuzzing

The previous steps should allow you to start fuzzing.

```bash
cd 
run `cargo r --bin fuzz_read_foo_file`
```
