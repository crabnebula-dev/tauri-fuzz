use env_logger;
use log::trace;
use mini_app::tauri_commands::basic::payload_for_tauri_cmd_2;
use mini_app::tauri_commands::shell::payload_for_bin_sh;
use mini_app::tauri_commands::shell::payload_for_shell_command_0;
#[allow(unused_imports)]
use mini_app::*;
use tauri::api::process::Command;

fn main() {
    env_logger::init();
    trace!("Start tracing");
    let data = 8u32.to_be_bytes();
    let app = setup_tauri_mock().expect("Failed to init Tauri app");
    call_tauri_cmd(app, payload_for_tauri_cmd_2(&data));
    let app = setup_tauri_mock().expect("Failed to init Tauri app");
    let data = "whoami".as_bytes();
    call_tauri_cmd(app, payload_for_bin_sh(data));

    // println!("{:?}", Command::new("ps").args(["-p", "$$"]).output());
}
