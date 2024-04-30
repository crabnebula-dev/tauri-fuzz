use log::trace;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

#[tauri::command]
/// Write to the file `assets/foo.txt`
pub fn write_foo_file(input: &str) {
    trace!("[write_foo_file] Entering with input: {}", input);
    let file_path = get_foo_path();
    let mut file = File::create(file_path.clone()).unwrap();
    file.write_all(input.as_bytes())
        .expect("Failed to write too foo file");
}

#[tauri::command]
/// Read the file `assets/foo.txt`
pub fn read_foo_file() -> String {
    trace!("[read_foo_file] Entering");
    let path = get_foo_path();
    let mut content = String::new();
    let mut file = File::open(path).unwrap();
    file.read_to_string(&mut content).unwrap();
    content
}

/// Get the path to `assets/foo.txt`
fn get_foo_path() -> PathBuf {
    let mut file_path = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    file_path.push("assets");
    file_path.push("foo.txt");
    file_path
}
