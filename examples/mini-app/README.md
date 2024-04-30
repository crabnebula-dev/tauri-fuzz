# Mini-app

This is a minimal Tauri app that is used for testing our fuzzer on Tauri applications.

## `mini-app` different tauri commands

- `basic.rs` contains tauri commands that can crash depending on the input
- `file_access.rs` contains Tauri commands that use the filesystem
- `libc_calls.rs` contains Tauri commands that calls the `libc` directly
- `shell.rs` contains Tauri commands that calls `shell` functionality

