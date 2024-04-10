# How to integrate fuzzing in your Tauri app

As an example we will show how we add fuzzing to the Tauri application `mini-app`.
In this app we fuzz the tauri command `mini-app::tauri_commands::file_access::read_foo_file`.

## Make your Tauri app both a binary and a crate

By default a Tauri application will just be a binary.
We want the Rust code to also be a crate so that the fuzzer is able to call the application
Tauri commands.

1. Add a `lib.rs` file in `mini-app/src-tauri/src`
2. In the `lib.rs` file make the Tauri commands you want to fuzz public
```rust,ignore
pub use tauri_commands::file_access::read_foo_file;
```

## Setup the `fuzz` directory which is the fuzzing environment

Copy the directory `mini-app-fuzz`

### Dependencies

Dependencies in `Cargo.toml` file should look like this

1. Add your Tauri app as a dependency
```toml
[dependencies]
// Your Tauri app
mini-app = { git = "ssh://git@github.com/crabnebula-dev/tauri-fuzzer.git", branch = "main" }
```


### Write your fuzz target

An example can be found in `mini-app-fuzz/fuzz_targets/read_foo_file.rs`.


With the function `mini-app::tauri_commands::file_access::read_foo_file`:
1. Copy the template from `mini-app-fuzz/fuzz_targets/template.rs`
2. Rename it with a name of your choice (e.g `fuzz_read_foo_file.rs`)
3. Fill the corresponding information.
    - command name is `read_foo_file`
    - in the harness, generate the Tauri app with the handle `mini-app::tauri_commands::file_access::read_foo_file`
    - fill the `create_payload` to invoke your Tauri command with the right parameters
    - specify the `FuzzPolicy` you want to apply

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


## Example `Cargo.toml` in the fuzz directory

```toml
[package]
# Your package name
name = "mini-app-fuzz"
version = "0.0.0"
publish = false
edition = "2021"

[dependencies]
# Your Tauri app
mini-app = { git = "ssh://git@github.com/crabnebula-dev/tauri-fuzzer.git", branch = "main" }

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

