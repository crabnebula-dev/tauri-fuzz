#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, you wrote {}!", name)
}
