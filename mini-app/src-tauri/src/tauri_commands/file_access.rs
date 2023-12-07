use log::trace;
use tempfile::tempdir;
use std::io::{Read, Write};
use std::fs::File;

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
