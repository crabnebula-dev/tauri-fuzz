// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

// This is a template to create a fuzz target
//
// Steps:
// 1. Copy this file and rename it
// 2. Change the target details below
// 3. Add the new fuzz target in [[bin]] table in Cargo.toml of your project
//
// Note: you may need to implement [FromRandomBytes] for your command argument types.

appfuzz_rt::fuzz_tauri_command! {
    // Name of the tauri command you want to fuzz
    command: "read_foo_file",
    // Pointer to the tauri command you want to fuzz
    path: mini_app::file_access::read_foo_file,
    // Parameters names and types to the tauri command
    parameters: {
        name: String,
    },
    // Policy chosen for the fuzzing
    // Here the policy will not allow any access to the filesystem
    policy: policies::filesystem::no_file_access(),
}
