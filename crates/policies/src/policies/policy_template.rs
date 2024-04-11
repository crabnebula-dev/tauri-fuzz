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
            // Ex: "open".into()    // for filesystem access
            name: todo!(),

            // Library in which the function monitored resides.
            // If it's a Rust crate, due to static linking the lib will
            // corresponds to the binary name
            // If it's libc it's a dynamic library you can give the libc name directly
            // Ex: "libc".into()   // "open" is in the libc library
            lib: todo!(),

            // Rule that the function will need to follow to respect the `FunctionPolicy`
            // Ex: Rule::OnEntry(block_on_entry)  // to block on function entry
            rule: Rule::OnEntry(block_on_entry),

            // Description used when an execution does not respect the rule specified above
            // Ex: "Access to function [open] denied"
            description: todo!(),

            // Number of parameters the function takes
            // Ex: 2    // The function "open" takes 2 parameters
            nb_parameters: todo!(),
        },
        // We can monitor any number of functions that can violate our security policy
        FunctionPolicy {
            name: todo!(),
            lib: todo!(),
            rule: todo!(),
            description: todo!(),
            nb_parameters: todo!(),
        },
    ]
}
