use env_logger;
use log::trace;
#[allow(unused_imports)]
use mini_app::*;
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri_fuzz_tools::{create_invoke_payload, invoke_command_and_stop, CommandArgs};

fn main() {
    env_logger::init();
    trace!("Start tracing");

    let app = mock_builder()
        .invoke_handler(tauri::generate_handler![
            mini_app::tauri_commands::file_access::write_read_tmp_file,
            mini_app::tauri_commands::file_access::read_foo_file,
            tauri_cmd_2,
            no_args,
            mini_app::direct_syscalls::write_to_stdout
        ])
        .build(mock_context(noop_assets()))
        .expect("Failed to init Tauri app");

    let mut args = CommandArgs::new();
    let mut path = std::env::current_dir().unwrap();
    path.push("test_assets");
    path.push("foo.txt");

    args.insert("path", path.to_str().unwrap());

    let payload = create_invoke_payload("read_foo_file", args);

    let res = invoke_command_and_stop::<String>(app, payload);
    println!("{:?}", res);
}
