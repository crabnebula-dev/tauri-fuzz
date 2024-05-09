mod fuzzer;
mod fuzzer_options;
pub use crate::fuzzer::{fuzz_main, fuzz_test};
pub use crate::fuzzer_options::SimpleFuzzerConfig;

#[cfg(feature = "tauri")]
pub mod tauri_utils;

#[macro_export]
macro_rules! define_fuzz_target {
    (
        command: $command:literal,
        path: $path:path,
        parameters: {
            $($param:ident : $param_fn:expr),+ $(,)?
        },
        policy: $policy:expr $(,)?
    ) => {
        use ::fuzzer::tauri_utils::{create_invoke_payload, invoke_command_minimal, CommandArgs};
        use ::libafl::inputs::{BytesInput, HasBytesVec};
        use ::libafl::prelude::ExitKind;
        use ::tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
        use ::tauri::App as TauriApp;
        use ::tauri::InvokePayload;

        const COMMAND_NAME: &str = $command;

        fn main() {
            let ptr = harness as *const ();
            let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
            let fuzz_config_file = fuzz_dir.join("fuzzer_config.toml");
            let options = fuzzer::SimpleFuzzerConfig::from_toml(fuzz_config_file, COMMAND_NAME, fuzz_dir).into();
            ::fuzzer::fuzz_main(harness, options, ptr as usize, $policy);
        }

        fn harness(input: &BytesInput) -> ExitKind {
            let app = mock_builder()
                .invoke_handler(tauri::generate_handler![$path])
                .build(mock_context(noop_assets()))
                .expect("Failed to init Tauri app");

            let _ = invoke_command_minimal(app, create_payload(input.bytes()));

            ExitKind::Ok
        }

        fn create_payload(bytes: &[u8]) -> InvokePayload {
            let mut params = CommandArgs::new();

            $(
                let param = $param_fn(bytes);
                params.insert(stringify!($param).to_string(), param);
            )*

            create_invoke_payload(None, COMMAND_NAME, params)
        }
    }
}
