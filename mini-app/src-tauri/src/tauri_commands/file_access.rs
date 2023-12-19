use log::trace;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use tempfile::tempdir;

#[tauri::command]
pub fn write_read_tmp_file(input: &str) -> String {
    trace!("[write_read_tmp_file] Entering with input: {}", input);
    let tmp_dir = tempdir().unwrap();
    let file_path = tmp_dir.path().join("foo.txt");
    {
        // Write
        let mut file = File::create(file_path.clone()).unwrap();
        file.write_all(input.as_bytes()).unwrap();
    }
    // Read
    let mut content = String::new();
    let mut file = File::open(file_path).unwrap();
    file.read_to_string(&mut content).unwrap();
    content
}

#[tauri::command]
pub fn read_foo_file(path: String) -> String {
    trace!("[read_foo_file] Entering");
    let path = PathBuf::from_str(&path).unwrap();
    let mut content = String::new();
    let mut file = File::open(path).unwrap();
    file.read_to_string(&mut content).unwrap();
    content
}
