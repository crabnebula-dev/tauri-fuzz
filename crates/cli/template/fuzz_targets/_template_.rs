// // Copyright 2024-2022 CrabNebula Ltd., Alexandre Dang
// // SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

// This is a template to create a fuzz target
//
// Steps:
// 1. Copy this file and rename it
// 2. Change the target details below
// 3. Add the new fuzz target in [[bin]] table in Cargo.toml of your project
//
// Note: you may need to implement [FromRandomBytes] for your command argument types.

fuzzer::define_fuzz_target! {
    command: "greet",
    path: {{crate_name_underscored}}::greet,
    parameters: {
        name: String,
    },
    policy: policies::filesystem::no_file_access(),
}
