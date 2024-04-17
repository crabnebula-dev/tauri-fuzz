use fuzzer::tauri_utils::{
    create_invoke_payload, invoke_command_minimal, mock_builder_minimal, CommandArgs,
};
use libafl::inputs::{BytesInput, HasBytesVec};
use libafl::prelude::ExitKind;
use tauri::test::{mock_context, noop_assets, MockRuntime};
use tauri::App as TauriApp;
use tauri::InvokePayload;

const COMMAND_NAME: &str = "read_foo_file";

fn setup_tauri_mock() -> Result<TauriApp<MockRuntime>, tauri::Error> {
    mock_builder_minimal()
        .invoke_handler(tauri::generate_handler![
            mini_app::tauri_commands::file_access::read_foo_file
        ])
        .build(mock_context(noop_assets()))
}

pub fn main() {
    let addr = mini_app::tauri_commands::file_access::read_foo_file as *const () as usize;
    let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
    let harness = |input: &BytesInput| {
        let app = setup_tauri_mock().expect("Failed to init Tauri app");
        let _res = invoke_command_minimal(app, create_payload(input.bytes()));
        ExitKind::Ok
    };

    fuzzer::fuzz_main(
        harness,
        options,
        addr,
        policies::file_policy::no_access_to_filenames(),
    );
}

fn create_payload(_bytes: &[u8]) -> InvokePayload {
    let args = CommandArgs::new();
    create_invoke_payload(None, COMMAND_NAME, args)
}

#[cfg(test)]
mod test {
    use super::*;

    // This is a trick to capture the fuzzer exit status code.
    // The fuzzer exits the process with an error code rather than panicking.
    // This test will be started as a new process and its exit status will be captured.
    #[test]
    #[ignore]
    fn hidden_read_foo_block_all_file_access() {
        let addr = mini_app::file_access::read_foo_file as *const ();
        let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
        let harness = |input: &BytesInput| {
            let app = setup_tauri_mock().expect("Failed to init Tauri app");
            let _res = invoke_command_minimal(app, create_payload(input.bytes()));
            ExitKind::Ok
        };
        unsafe {
            let _ = fuzzer::fuzz_test(
                harness,
                &options,
                addr as usize,
                policies::file_policy::no_file_access(),
            )
            .is_ok();
        }
    }

    // This is a trick to capture the fuzzer exit status code.
    // The fuzzer exits the process with an error code rather than panicking.
    // This test will be started as a new process and its exit status will be captured.
    #[test]
    #[ignore]
    fn hidden_read_foo_block_access_to_foo() {
        let addr = mini_app::file_access::read_foo_file as *const ();
        let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
        let harness = |input: &BytesInput| {
            let app = setup_tauri_mock().expect("Failed to init Tauri app");
            let _res = invoke_command_minimal(app, create_payload(input.bytes()));
            ExitKind::Ok
        };
        unsafe {
            let _ = fuzzer::fuzz_test(
                harness,
                &options,
                addr as usize,
                policies::file_policy::no_access_to_filenames(),
            )
            .is_ok();
        }
    }

    // Block reading foo with no file access policy
    #[test]
    fn read_foo_block_all_file_access() {
        let exe = std::env::current_exe().expect("Failed to extract current executable");
        let status = std::process::Command::new(exe)
            .args(&["--ignored", "hidden_read_foo_block_all_file_access"])
            .status()
            .expect("Unable to run program");

        if cfg!(target_os = "windows") {
            // Check that fuzzer process launched exit with status error 1
            assert_eq!(Some(1), status.code());
        } else {
            // Check that fuzzer process launched exit with status error 134
            assert_eq!(Some(134), status.code());
        }
    }

    // Block reading foo with no access to files with name "foo.txt"
    #[test]
    fn read_foo_block_access_by_filename() {
        let exe = std::env::current_exe().expect("Failed to extract current executable");
        let status = std::process::Command::new(exe)
            .args(&["--ignored", "hidden_read_foo_block_access_to_foo"])
            .status()
            .expect("Unable to run program");

        if cfg!(target_os = "windows") {
            // Check that fuzzer process launched exit with status error 1
            assert_eq!(Some(1), status.code());
        } else {
            // Check that fuzzer process launched exit with status error 134
            assert_eq!(Some(134), status.code());
        }
    }

    // Read-only access should not block `read_foo`
    #[test]
    fn read_foo_accepted_by_readonly_policy() {
        let addr = mini_app::file_access::read_foo_file as *const ();
        let fuzz_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
        let options = fuzzer::get_fuzzer_options(COMMAND_NAME, fuzz_dir);
        let harness = |input: &BytesInput| {
            let app = setup_tauri_mock().expect("Failed to init Tauri app");
            let _res = invoke_command_minimal(app, create_payload(input.bytes()));
            ExitKind::Ok
        };
        unsafe {
            let _ = fuzzer::fuzz_test(
                harness,
                &options,
                addr as usize,
                policies::file_policy::read_only_access(),
            )
            .is_ok();
        }
    }
}
