// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

/// A template to create a `FuzzPolicy`

// A function that will create our `FuzzPolicy` at runtime
pub fn no_file_access() -> FuzzPolicy {
    // A `FuzzPolicy` is a vector of `FunctionPolicy`.
    //
    // A `FunctionPolicy` will attached itself on a function and its
    // rule will be checked when executing the function.
    vec![
        FunctionPolicy {
            // Name of the function monitored
            name: "open".into(),

            // Library in which the function monitored resides.
            // If it's a Rust crate, due to static linking the lib will
            // corresponds to the binary
            // If it's libc it's a dynamic library you can give the libc name directly
            lib: LIBC.into(),

            // Rule that the function will need to follow to respect the `FunctionPolicy`
            rule: Rule::OnEntry(block_on_entry),

            // Description used when an execution does not respect the rule specified above
            description: "Access to [fopen] denied".into(),

            // Number of parameters the function takes
            nb_parameters: 2,

            // Specify if we are monitoring a Rust function
            is_rust_function: false,
        },
        // We also monitor a second function that can violate our security policy
        FunctionPolicy {
            name: "open64".into(),
            lib: LIBC.into(),
            rule: Rule::OnEntry(block_on_entry),
            description: "Access to [open64] denied".into(),
            nb_parameters: 2,
            is_rust_function: false,
        },
    ]
}
