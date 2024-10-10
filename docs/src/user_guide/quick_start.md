# Quick Start

## Goal

We will fuzz a very minimal Tauri application.
The repository features a minimal example called [`mini-app`](https://github.com/crabnebula-dev/tauri-fuzzer/tree/main/examples/mini-app).
This example app will be used to showcase how to setup fuzzing with `AppFuzz Runtime`.

## Fuzzing a Tauri application

We are using `mini-app` that implements a simple Tauri command, that will later be fuzzed.

<details>
<summary>
Tauri app structure
</summary>

```ignore
mini-app
- ...
- src/
- src-tauri/
    - src/
        - lib.rs
        - main.rs
        - tauri_commands/
            - file_access.rs
            - read_foo_file
            - ...
    - Cargo.toml
```

</details>

### Make the Tauri Application Accessible to the Fuzzer

The Tauri app backend must be compiled as a crate such that the Tauri commands are **exposed** to the fuzzer.

For example we want to fuzz the Tauri commands called `read_foo_file`:

<details>
<summary> `mini-app/src-tauri/Cargo.toml` </summary>

```toml,ignore
[package]
name = "mini-app"
version = "0.0.0"
description = "A Tauri App"

# This section is automatic in Tauri v2
[lib]
crate-type = ["staticlib", "cdylib", "rlib"]
```

</details>

<details>
<summary> `mini-app/src-tauri/lib.rs` </summary>

```rust,ignore
/// define the module
pub mod tauri_commands;

/// publicly re-export the Taur command `read_foo_file`
pub use tauri_commands::file_access::read_foo_file
```

</details>

<details>
<summary>
`mini-app/src-tauri/src-tauri/tauri_commands/file_access.rs`
</summary>

```rust,ignore
#[tauri::command]
/// Mark the function as public
pub fn read_foo_file() -> String {
    let path        = get_foo_path();
    let mut content = String::new();
    let mut file    = File::open(path).unwrap();
    file.read_to_string(&mut content).unwrap();
    content
}
```

</details>

### Fuzzing our Tauri app, quick guide

> This section requires the CLI utility `cargo-tauri-fuzz`
> The project contains a CLI package `cargo-tauri-fuzz` that helps setting up fuzzing for your Tauri app.
> The CLI package resides in `crates/cli`.
> If any issue arises from using the CLI we recommend you follow the [manual steps guide](manual_fuzzing.md)

#### 1. Create fuzz directory

Execute `cargo-tauri-fuzz init` in `mini-app/src-tauri`.

<details>
<summary>
Tauri app structure with fuzz directory
</summary>

```ignore
Project
- ...
- src/
  - ...
- src-tauri/
    - src/
        - lib.rs
        - main.rs
        - tauri_commands/
            - file_access.rs
            - read_foo_file
            - ...
    - fuzz/
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

#### 2. Write your fuzz target

- Copy `mini-app/src-tauri/fuzz/fuzz_targets/_template_.rs` as `mini-app/src-tauri/fuzz/fuzz_targets/fuzz_read_foo.rs`
- Fill `mini-app/src-tauri/fuzz/fuzz_targets/fuzz_read_foo.rs` with relevant information

<details>
<summary>
`mini-app/src-tauri/fuzz/fuzz_targets/fuzz_read_foo.rs`

</summary>

Here we will fuzz the Tauri command `read_foo` against a policy that does not allow any file access.

```rust,ignore
{{#include ../../../crates/cli/template/fuzz_targets/_template_.rs}}
```

</details>

#### 3. Add the fuzz target as binary

Add `fuzz_read_foo` as a binary in `mini-app/src-tauri/fuzz/Cargo.toml`

<details>
<summary>
`mini-app/src-tauri/fuzz/Cargo.toml`
</summary>

```toml,ignore
{{#include ../../../crates/cli/template/Cargo.crate-manifest}}
```

</details>

#### 4. Start fuzzing

Start fuzzing by executing one of these commands.

From `mini-app/src-tauri/` directory:

```bash
cargo-tauri-fuzz fuzz fuzz_read_foo
```

From `mini-app/src-tauri/fuzz/` directory:

```bash
cargo r --bin fuzz_read_foo
```
