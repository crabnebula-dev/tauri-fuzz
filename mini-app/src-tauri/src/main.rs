use env_logger;
use log::trace;
#[allow(unused_imports)]
use mini_app::*;

fn main() {
    env_logger::init();
    trace!("Start tracing");
    // let data = 8u32.to_be_bytes();
    // let app = setup_tauri_mock().expect("Failed to init Tauri app");
    // fuzz::invoke_tauri_cmd(app, payload_for_tauri_cmd_2(&data));
    // let app = setup_tauri_mock().expect("Failed to init Tauri app");
    // let data = "whoami".as_bytes();
    // call_tauri_cmd(app, payload_for_bin_sh(data));

    // let data = "whoamia".as_bytes();
    // let code = bin_sh(data);
    // println!("code: {:?}", code)

    // println!("{:?}", Command::new("ps").args(["-p", "$$"]).output());
}
