/// This is a template to create a fuzz target
///
/// Steps:
/// 1. Copy this file and rename it
/// 2. Change the target details below
/// 3. Add the new fuzz target in [[bin]] table in Cargo.toml of your project

fuzzer::define_fuzz_target! {
    command: "greet",
    path: {{crate_name_underscored}}::greet,
    parameters: {
        name: |bytes: &[u8]| String::from_utf8_lossy(bytes).to_string(),
    },
    policy: policies::file_policy::no_file_access(),
}
