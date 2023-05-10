#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[tauri::command]
fn dangerous_func(input: &str) -> String {
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![dangerous_func])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
