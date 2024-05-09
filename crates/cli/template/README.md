# Fuzzing with Frida

All the commands presented here have to be run from within directory.

## Fuzz

### What is a fuzz target

A _fuzz_target_ is a block of code or harness that the fuzzer will execute repeatedly
with different inputs.
In this project each _fuzz_target_ corresponds to a Tauri command of the application.

### List all fuzz targets

Each of the files in the `fuzz_targets/` directory corresponds to a fuzz target.
Two ways to get the fuzz targets:

- Check the file names in `fuzz_targets/` without the `*.rs` extension
- Go to the `fuzz/Cargo.toml` file and check the `[bin]` sections

### Fuzz a fuzz target

- run `cargo r --bin [fuzz target]`

#### Check solutions

- Fuzzing solutions are available in `fuzz_solutions/[fuzz target]_solutions/`
- check solutions with the files in: `fuzz_solutions/[fuzz target]_solutions/file_name`
  - for `tauri_cmd_1` it should contain the value `abc`
  - for `tauri_cmd_2` it should contain the value `100`
    - with `xxd tauri_cmd_2_solutions/file_name` it should contain `0x0064`

### Create your fuzz target

You can copy-paste the `fuzz_targets/template.rs` and fill it with your implementation
