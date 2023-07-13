use mini_app::*;

fn main() {
    // let app = setup_tauri_mock();
    // call_tauri_cmd_1(app);
    let app = setup_tauri_mock();
    let input = 3;
    call_tauri_cmd_2(app, input);
    // let app = setup_tauri_mock();
    // let input = 100;
    // call_tauri_cmd_2(app, input);
}
