# Fuzzer

All the commands presented here have to be run from within directory.

## Fuzz

### What is a fuzz target

A _fuzz_target_ is a block of code or harness that the fuzzer will execute repeatedly
with different inputs.
In this project each _fuzz_target_ corresponds to a Tauri command of the application.

### Create your fuzz target

#### Create the fuzz target code

You can copy-paste the `fuzz_targets/_template_.rs` or `fuzz_targets/_template_full_.rs` and modify them
to call the Tauri commands of your app.

- `fuzz_targets/_template_.rs` uses a macro to create the fuzz target it is the simplest option
- `fuzz_targets/_template_full_.rs` is a rendered version of the macro. You can use it if you need
  fine-grained customization of the fuzzing process.

#### Declare the fuzz target as a binary

**src-tauri/fuzz/Cargo.toml:**

```
[[bin]]
name = "{fuzz_target name}"
path = "fuzz_targets/{fuzz_target file}"
doc = false
```

### Fuzz a fuzz target

`cargo r --bin [fuzz_target]`
or
`cargo-tauri-fuzz fuzz [fuzz_target]`

#### Check solutions

Fuzzing solutions are available in `fuzz_solutions/[fuzz target]_solutions/`
Check solutions with the files in: `fuzz_solutions/[fuzz target]_solutions/file_name`
