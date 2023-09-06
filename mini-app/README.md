# Mini-app

This is a minimal Tauri app that is used for testing Tauri fuzzing.

## Steps to fuzz

1. Install `cargo-fuzz`
2. `cd` to "mini-app/src-tauri"
3. `cargo-fuzz list`
4. `cargo-fuzz run {one of the target of previous step}`, it needs +nightly compiler



