use env_logger;
#[allow(unused_imports)]
use mini_app::*;

fn main() {
    env_logger::init();
    let data = 8u32.to_be_bytes();
    let app = setup_tauri_mock().expect("Failed to init Tauri app");
    call_tauri_cmd(app, payload_for_tauri_cmd_2(&data));
}
