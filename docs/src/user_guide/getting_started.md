# Getting Started

## Goal

We will create a very minimal Tauri application, where we will configure a function entry point to be fuzzed with the tauri-fuzzer. The fuzzer repo features a minimal example called `mini-app`, that will be used to showcase aquick introduction to the tauri-fuzzer.

## Prerequisistes

This quickstart works with Tauri 1.x. and requires some modification. The Tauri 2.x version will have relaxed requirements.

- [System Dependencies](https://tauri.app/v1/guides/getting-started/prerequisites)
- [Rust](https://www.rust-lang.org)

There is a [template app](https://github.com/crabnebula-dev/tauri-fuzzer/tree/main/examples/mini-app), that provides some setup of a testing application.

## Steps

We are creating a small application that implements a simple Tauri command, that will later be fuzzed.

<details>
<summary>
Final project structure
</summary>

```ignore
Project
- ...
- src-tauri
    - src
        - lib.rs
        - main.rs
        - tauri_commands
            - file_access.rs
    - fuzzer
        - lib.rs
    - fuzz_read_foo_file
        -
    - Cargo.toml

```

</details>

### Preparing the Application

> Adding a library is only needed for Tauri v1.x. The next generation of Tauri will have this as a default application setup. Mobile applications need Tauri to be a library.

We will prepare a mini app to fuzz a very simple Tauri command. In order to do this, we need to **expose** the command to be fuzzed. Create, or modify the `lib.rs` file in the `src/` folder of the `mini-app` project. We want to provide a command to be fuzzed.

`lib.rs`

```rust,ignore
/// define the module
pub mod tauri_commands;

/// publicly re-export the Taur command `read_foo_file`
pub use tauri_commands::file_access::read_foo_file
```

The main application that will invoke the file_access command

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

and finally the file_access command implementation

`mini-app/src-tauri/src/tauri_commands/file_access.rs`:

```rust,ignore
use log::trace;

#[tauri::command]
/// Read the file `assets/foo.txt`
pub fn read_foo_file() -> String {
    trace!("[read_foo_file] Entering");
    let path        = get_foo_path();
    let mut content = String::new();
    let mut file    = File::open(path).unwrap();
    file.read_to_string(&mut content).unwrap();
    content
}
```

### Preparing the Fuzzer Application

Create a `fuzz` directory inside your `src-tauri` folder and copy the directory `mini-app-fuzz` into the `src-tauri/fuzz` directory of your application.

```bash
mkdir -p mini-app/src-tauri/fuzz
cp -r mini-app-fuzz/* mini-app/src-tauri/fuzz
cd mini-app/src-tauri/fuzz
```

#### Dependencies

Add your Tauri app as a dependency in the `mini-app/src-tauri/fuzz/mini-app-fuzz/Cargo.toml` file.
It should look like this:

```toml
[dependencies]
// Your Tauri app
mini-app            = { path = "../" }
```

When using the fuzzer inside of this fuzzer repository workspace the following dependencies can be configured
to use the workspace:

```toml
tauri               = { workspace = true, features = ["api-all"] }
tauri-utils         = { workspace = true }
tauri_fuzz_tools    = { workspace = true }
libafl              = { workspace = true }
libafl_bolts        = { workspace = true }
libafl_frida        = { workspace = true }
libafl_targets      = { workspace = true }
frida-gum           = { workspace = true }
```

<details>
<summary>
Example `Cargo.toml`
</summary>

```toml
[package]
# Your package name
name                = "mini-app-fuzz"
version             = "0.0.0"
publish             = false
edition             = "2021"

[dependencies]
# Your Tauri app
mini-app            = { path = "../" }

# Logging
log                 = "0.4"
env_logger          = "*"
color-backtrace     = "0.5"

# Frida binary analyser
frida-gum           = { version = "0.13.2", features = ["auto-download", "event-sink", "invocation-listener", ] }

# Our fork of LibAFL
libafl              = { git = "ssh://git@github.com/crabnebula-dev/LibAFL.git", features = [
  "std",
  "llmp_compression",
  "llmp_bind_public",
  "frida_cli",
], branch = "tauri" } #,  "llmp_small_maps", "llmp_debug"]}
libafl_bolts        = { git = "ssh://git@github.com/crabnebula-dev/LibAFL.git", branch = "tauri" }
libafl_frida        = { git = "ssh://git@github.com/crabnebula-dev/LibAFL.git", features = ["cmplog"], branch = "tauri" }
libafl_targets      = { git = "ssh://git@github.com/crabnebula-dev/LibAFL.git", features = ["sancov_cmplog"], branch = "tauri" }

# Official Tauri
tauri               = { version = "1.5", default-features = false, features = ["test", "tracing"] }
tauri-utils         = "1.5"

# Utility crate to connect the fuzzer and Tauri
tauri_fuzz_tools    = { git = "ssh://git@github.com/crabnebula-dev/tauri-fuzzer.git", branch = "main" }

# Fuzzer and policy used
[lib]
name                = "fuzzer"
path                = "fuzzer/lib.rs"

# Your fuzz target
[[bin]]
name                = "fuzz_read_foo_file"
path                = "fuzz_targets/fuzz_read_foo_file.rs"
doc                 = false
```

</details>

### Writing a Fuzz Target

We will finally create our fuzz target. We are going modify the template file provided by

<details>
<summary>
`crates/cli/template/fuzz_targets/_template_full_.rs` 
</summary>

```rust,ignore
{{#include ../../../crates/cli/template/fuzz_targets/_template_full_.rs}}
```

(_If you see no code, then the docs have to be modified_)

</details>

Next steps:

- Fill COMMAND_NAME with `read_foo_file`

```rust,ignore
const COMMAND_NAME: &str = "read_foo_file";
```

- in the harness, generate the Tauri app with the handle
  `mini-app::tauri_commands::file_access::read_foo_file`

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

## Providing an Example Target

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

## Start Fuzzing

The previous steps should allow you to start fuzzing.

```bash
cd
run `cargo r --bin fuzz_read_foo_file`
```
