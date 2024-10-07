// // Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// // SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

mod greet;
pub use crate::greet::greet;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .setup(move |app| {
            tauri::WebviewWindowBuilder::new(app, "main", Default::default())
                .build()
                .unwrap();
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Failed to init Tauri app");
    app.run(|_, _| {});
}
