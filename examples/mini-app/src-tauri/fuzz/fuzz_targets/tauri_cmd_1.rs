::fuzzer::define_fuzz_target! {
    command: "tauri_cmd_1",
    path: mini_app::basic::tauri_cmd_1,
    parameters: {
        input: |bytes: &[u8]| String::from_utf8_lossy(&bytes).to_string(),
    },
    policy: policies::file_policy::no_file_access(),
}
