# fuzzer-cli

We created a cli to facilitate the setting up the fuzzing environment of your Tauri application.

## Setup the fuzz directory

Create the `src-tauri/fuzz/` directory in your Tauri app backend code `src-tauri/`.

```ignore
cargo-tauri-fuzz init
```

## Fuzz

Fuzz a target that is specified in `src-tauri/fuzz/Cargo.toml`

```ignore
cargo-tauri-fuzz fuzz [fuzz_target]
```

## Analyze the fuzz results

Check the results in `src-tauri/fuzz/fuzz_solutions/[fuzz target]_solutions/`.
