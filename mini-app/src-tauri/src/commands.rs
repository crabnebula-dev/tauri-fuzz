#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![allow(unused_imports)]

#[tauri::command]
#[no_mangle]
// No mangle is used for fuzzing when using Qemu
pub fn tauri_cmd_1(input: &str) -> String {
    println!("[mini-app] Entering tauri_cmd_1 with input:\n{}", input);
    let mut bytes = input.bytes();
    if bytes.next() == Some(b'a') {
        if bytes.next() == Some(b'b') {
            if bytes.next() == Some(b'c') {
                panic!("[mini-app] Crashing! =)");
            }
        }
    }
    println!("[mini-app] Exiting tauri_cmd_1");
    format!("Hello, you wrote {}!", input)
}

#[tauri::command]
#[no_mangle]
// No mangle is used for fuzzing when using Qemu
pub fn tauri_cmd_2(input: u32) -> String {
    println!("[mini-app] Entering tauri_cmd_2 with input:\n{}", input);
    if input == 100 {
        panic!("[mini-app] Crashing! =)");
    }
    println!("[mini-app] Exiting tauri_cmd_2");
    format!("Hello, you wrote {}!", input)
}
