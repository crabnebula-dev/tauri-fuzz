# Manual Steps to Fuzz your Tauri App

This is an extension to the [Quick Start guide][quick_start.md].
The different steps to fuzz a Tauri app are detailed here.

We will fuzz a very minimal Tauri application.
The repository features a minimal example called [`mini-app`](https://github.com/crabnebula-dev/tauri-fuzzer/tree/main/examples/mini-app).
This example app will be used to showcase how to setup fuzzing with `AppFuzz Runtime`.

## Prepare your Tauri App

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

{{#include ./prepare_tauri_app.txt}}

## Create the application fuzz package

We will obtain this project structure:

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

### With the CLI `cargo-tauri-fuzz`

> This section requires the CLI utility `cargo-tauri-fuzz`
> The project contains a CLI package `cargo-tauri-fuzz` that helps setting up fuzzing for your Tauri app.
> The CLI package resides in `crates/cli`.
> If any issue arises from using the CLI we recommend you read the next section

Execute `cargo-tauri-fuzz init` in `mini-app/src-tauri`.

### Setup the fuzz directory manually

You can copy-paste the example in the [repo](https://github.com/crabnebula-dev/tauri-fuzzer/tree/main/examples/mini-app/src-tauri/fuzz).

1. Create the fuzz directory

```bash
mkdir -p mini-app/src-tauri/fuzz
```

2. Add Cargo.toml in the fuzz directory

<details>
<summary>
`mini-app/src-tauri/fuzz/Cargo.toml`
</summary>

```toml, ignore
{{#include ../../../crates/cli/template/Cargo.crate-manifest}}
```

</details>

3. Add fuzz_targets directory with templates

```bash
mkdir -p mini-app/src-tauri/fuzz/fuzz_targets
touch mini-app/src-tauri/fuzz_targets/_template_.rs
touch mini-app/src-tauri/fuzz_targets/_template_full_.rs
```

<details>
<summary>
`mini-app/src-tauri/fuzz/fuzz_targets/_template_.rs`
</summary>

```toml, ignore
{{#include ../../../crates/cli/template/fuzz_targets/_template_.rs}}
```

</details>

<details>
<summary>
`mini-app/src-tauri/fuzz/fuzz_targets/_template_full_.rs`
</summary>

```toml, ignore
{{#include ../../../crates/cli/template/fuzz_targets/_template_full_.rs}}
```

</details>

4. Add `build.rs` and `tauri.conf.json`

<details>
<summary> `mini-app/src-tauri/fuzz/build.rs` </summary>

```toml,ignore
{{#include ../../../crates/cli/template/build.rs}}
```

</details>

<details>
<summary> `mini-app/src-tauri/fuzz/tauri.conf.json` </summary>

```toml,ignore
{{#include ../../../crates/cli/template/tauri.conf.json}}
```

</details>

## Writing a Fuzz Target

We will finally create our fuzz target.
We fuzz the Tauri commands `read_foo_file` which tries to read the file `foo.txt`.
The fuzz policy that we will choose is `policies::file_policy::no_file_access()`
that do not allow access to the filesystem.

There are two ways to write the fuzz target:

- with a Rust macro `fuzz_tauri_command`
- manually by filling the template

### Fill the template with macro

{{#include ./write_fuzz_target_with_macro.txt}}

> [!Disclaimer]
> Our macro is not stable yet it may not work for complex cases.
> For more control over the fuzzing we suggest that you write the fuzz target manually by following the next section.

### Fill the template with no macro

We are going to copy and modify the template file provided by

<details>
<summary>
`crates/cli/template/fuzz_targets/_template_full_.rs` 
</summary>

```rust,ignore
{{#include ../../../crates/cli/template/fuzz_targets/_template_full_.rs}}
```

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

- specify the [policy](./available_policies.md) you want to apply

```rust,ignore
policies::file_policy::no_file_access()
```

## Start Fuzzing

{{#include ./start_fuzzing.txt}}

## Validate Fuzzing Results

{{#include ./fuzzing_result.txt}}
