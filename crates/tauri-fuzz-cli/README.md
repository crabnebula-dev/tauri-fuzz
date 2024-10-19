# fuzzer-cli

We created a cli to facilitate the setting up the fuzzing environment of your Tauri application.
This is a guide on how to fuzz a Tauri app using the cli.

## tl;dr

#### 1. Make your Tauri app fuzz-compatible

- make your Tauri app as a crate
- make your Tauri commands a

#### 2. Setup the fuzz directory

```ignore
cargo-tauri-fuzz init
```

#### 3. Create your fuzz target in the fuzz directory

Copy the templates and fill them with your Tauri commands info.

#### 4. Fuzz

```ignore
cargo-tauri-fuzz fuzz [fuzz_target]
```

#### 5. Analyze the fuzz results

Check the results in `src-tauri/fuzz/fuzz_solutions/[fuzz target]_solutions/`.

## Step-by-step Guide

### Make your Tauri app fuzz-compatible

The principle of this fuzzer is to call the different Tauri commands of your app from the fuzzer.
For this to be possible the Tauri app should be compiled as a crate and the Tauri commands should
be visible and accessible to the fuzzer.

#### Tauri app as a crate

This should be the default if you're using Tauri 2.
_Cargo.toml_:

```ignore
[lib]
crate-type = ["staticlib", "cdylib", "rlib"]
```

#### Tauri commands

The tauri commands of your app should be visible and accessible from outside the app.

```ignore
#[tauri::command]
pub fn greet(name: &str) -> String {
  ...
}
```

**Warning**
There is a known issue in Tauri that a Tauri command can't be both be defined in the `lib.rs` and be public.
To solve this situation we recommend defining your Tauri commands in a separate file.

### Setup the fuzz directory

In your Tauri app project you can run

```ignore
cargo-tauri-fuzz init
```

This will create a fuzz folder in your Tauri directory: `src-tauri/fuzz/`.

### Create your fuzz target in the fuzz directory

#### What is a fuzz target

A _fuzz_target_ is a block of code or harness that the fuzzer will execute repeatedly
with different inputs.
In this project each _fuzz_target_ corresponds to a Tauri command that will be fuzzed by your app

#### Create your fuzz target

You can copy-paste the `fuzz_targets/_template_.rs` or `fuzz_targets/_template_full_.rs` and modify them
to call the Tauri commands of your app.

- `fuzz_targets/_template_.rs` uses a macro to create the fuzz target it is the simplest option
- `fuzz_targets/_template_full_.rs` is a rendered version of the macro. You can use it if you need
  fine-grained customization of the fuzzing process.

#### Declare the fuzz target as a binary

**src-tauri/fuzz/Cargo.toml:**

```ignore
[[bin]]
name = "[fuzz_target name]"
path = "fuzz_targets/[fuzz_target file]"
doc = false
```

### Fuzz a fuzz target

`cargo-tauri-fuzz fuzz [fuzz_target]`
or
`cargo r --bin [fuzz_target]` in the fuzz directory

### Analyze the fuzz results

Fuzzing solutions are available in the following folder: `src-tauri/fuzz/fuzz_solutions/[fuzz target]_solutions/`
