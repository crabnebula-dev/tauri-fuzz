#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[tauri::command]
#[no_mangle]
fn tauri_cmd_1(input: &str) -> String {
    println!("Entering tauri_cmd_1 with input:\n{}", input);
    let mut bytes = input.bytes();
    if bytes.next() == Some(b'a') {
        if bytes.next() == Some(b'b') {
            if bytes.next() == Some(b'c') {
                panic!("Crashing! =)");
            }
        }
    }
    format!("Hello, you wrote {}!", input)
}

#[tauri::command]
#[no_mangle]
fn tauri_cmd_2(input: u32) -> String {
    println!("Entering tauri_cmd_2 with input:\n{}", input);
    if input == 100 {
        panic!("Crashing! =)");
    }
    format!("Hello, you wrote {}!", input)
}


fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![tauri_cmd_1, tauri_cmd_2])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
