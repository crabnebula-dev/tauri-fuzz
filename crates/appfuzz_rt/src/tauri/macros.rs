#[macro_export]
macro_rules! fuzz_tauri_command {
    (
        command: $command:literal,
        path: $path:path,
        parameters: {
            $($param:ident : $param_type:ty),+ $(,)?
        },
        policy: $policy:expr $(,)?
    ) => {
        use appfuzz_rt::tauri::{create_invoke_request, invoke_command_minimal, CommandArgs};
        use appfuzz_rt::SimpleFuzzerConfig;
        use libafl::inputs::{BytesInput, HasMutatorBytes};
        use libafl::prelude::ExitKind;
        use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
        use tauri::webview::InvokeRequest;
        use tauri::WebviewWindow;


        const COMMAND_NAME: &str = $command;

        fn main() {
            let fuzz_dir = ::std::path::PathBuf::from(::std::env!("CARGO_MANIFEST_DIR"));
            let fuzz_config_file = fuzz_dir.join("fuzzer_config.toml");
            let options = SimpleFuzzerConfig::from_toml(fuzz_config_file, COMMAND_NAME, fuzz_dir).into();
            ::appfuzz_rt::fuzz_main(harness, &options, harness as *const () as usize, $policy, false);
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
                let param: $param_type = <$param_type as ::appfuzz_rt::tauri::FromRandomBytes>::from_random_bytes(bytes).unwrap();
                params.insert(stringify!($param).to_string(), param);
            )*

            create_invoke_request(None, COMMAND_NAME, params)
        }
    }
}
