#[tauri::command]
pub fn geteuid() -> u32 {
    // unsafe { libc::geteuid() }
    4
}
