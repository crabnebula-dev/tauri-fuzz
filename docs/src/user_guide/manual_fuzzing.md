# Manual Steps to Fuzzing

### Fuzzing our Tauri app manual steps

### Creating the application fuzz directory

After this step your project should be structured like this:

<details>
<summary>
Tauri app structure with fuzz directory
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
            - read_foo_file
            - ...
    - fuzz
        - build.rs
        - Cargo.toml
        - fuzz_targets/
            - _template_.rs
            - _template_full_.rs
        - fuzzer_config.toml
        - README.md
        - tauri.conf.json
    - Cargo.toml
```

</details>

#### One-liner with CLI

The project contains a CLI package `cargo-tauri-fuzz` that can initialize your fuzz directory in `crates/cli`.

Execute `$ cargo-tauri-fuzz init` in `mini-app/src-tauri`.

#### Manual steps

1. Create the fuzz directory

```bash
mkdir -p mini-app/src-tauri/fuzz
```

2. Add Cargo.toml in the fuzz directory

<details>
<summary>
Example `mini-app/src-tauri/fuzz/Cargo.toml`
</summary>

```toml, ignore
{{#include ../../../crates/cli/template/Cargo.crate-manifest}}
```

3. Add fuzz_targets directory with templates

```bash
mkdir -p mini-app/src-tauri/fuzz/fuzz_targets
touch mini-app/src-tauri/fuzz_targets/_template_.rs
touch mini-app/src-tauri/fuzz_targets/_template_full_.rs
```

<details>
<summary>
Example `mini-app/src-tauri/fuzz/fuzz_targets/_template_.rs`
</summary>

```toml, ignore
{{#include ../../../crates/cli/template/fuzz_targets/_template_.rs}}
```

<details>
<summary>
Example `mini-app/src-tauri/fuzz/fuzz_targets/_template_full_.rs`
</summary>

```toml, ignore
{{#include ../../../crates/cli/template/fuzz_targets/_template_full_.rs}}
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
