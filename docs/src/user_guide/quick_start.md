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

{{#include ./prepare_tauri_app.txt}}

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

{{#include ./write_fuzz_target_with_macro.txt}}

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

{{#include ./start_fuzzing.txt}}

#### 5. Check your solutions

{{#include ./fuzzing_result.txt}}
