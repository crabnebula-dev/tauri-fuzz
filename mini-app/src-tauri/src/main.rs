use env_logger;
use log::trace;
#[allow(unused_imports)]
use mini_app::*;
use tauri::test::{create_invoke_payload, CommandArgs};
use tauri::test::{mock_builder, mock_context, noop_assets};

fn main() {
    env_logger::init();
    trace!("Start tracing");

    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            tauri_cmd_2,
            no_args,
            mini_app::direct_syscalls::write_to_stdout
        ])
        .build(mock_context(noop_assets()))
        .expect("Failed to init Tauri app");

    let mut args = CommandArgs::new();
    args.insert("s", "toto");

    let payload = create_invoke_payload(String::from("write_to_stdout"), args);

    let _res = tauri::test::invoke_command_and_stop::<i64>(app, payload);
}
