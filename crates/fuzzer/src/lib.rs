mod fuzzer;
mod fuzzer_options;
pub use crate::fuzzer::{fuzz_main, fuzz_test};
pub use crate::fuzzer_options::SimpleFuzzerConfig;

#[cfg(feature = "tauri")]
pub mod tauri;

mod from_random_bytes;

pub use from_random_bytes::FromRandomBytes;

#[macro_export]
macro_rules! define_fuzz_target {
    (
        command: $command:literal,
        path: $path:path,
        parameters: {
            $($param:ident : $param_type:ty),+ $(,)?
        },
        policy: $policy:expr $(,)?
    ) => {
        use fuzzer::tauri_utils::{create_invoke_request, invoke_command_minimal, CommandArgs};
        use fuzzer::SimpleFuzzerConfig;
        use libafl::inputs::{BytesInput, HasBytesVec};
        use libafl::prelude::ExitKind;
        use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
        use tauri::webview::InvokeRequest;
        use tauri::WebviewWindow;


        const COMMAND_NAME: &str = $command;

        fn main() {
            let fuzz_dir = ::std::path::PathBuf::from(::std::env!("CARGO_MANIFEST_DIR"));
            let fuzz_config_file = fuzz_dir.join("fuzzer_config.toml");
            let options = SimpleFuzzerConfig::from_toml(fuzz_config_file, COMMAND_NAME, fuzz_dir).into();
            ::fuzzer::fuzz_main(harness, &options, harness as *const () as usize, $policy, false);
        }

        fn setup_mock() -> WebviewWindow<MockRuntime> {
            let app = mock_builder()
                .invoke_handler(tauri::generate_handler![$path])
                .build(mock_context(noop_assets()))
                .expect("Failed to init Tauri app");
            let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
                .build()
                .unwrap();
            webview
        }


        fn harness(input: &BytesInput) -> ExitKind {
            let webview = setup_mock();
            let _ = invoke_command_minimal(webview, create_request(input.bytes()));
            ExitKind::Ok
        }

        fn create_request(bytes: &[u8]) -> InvokeRequest {
            let mut params = CommandArgs::new();

            $(
                let param: $param_type = <$param_type as ::fuzzer::FromRandomBytes>::from_random_bytes(bytes).unwrap();
                params.insert(stringify!($param).to_string(), param);
            )*

            create_invoke_request(None, COMMAND_NAME, params)
        }
    }
}
