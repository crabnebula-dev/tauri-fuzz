#![allow(unused_imports)]
use log::trace;
use tauri::api::process::Command;
use tauri::api::shell::Program;

#[tauri::command]
#[no_mangle]
pub fn open_injection(input: String) {
    trace!("[open_injection] Entering with input: {:?}", input);
    unimplemented!()
}

#[tauri::command]
#[no_mangle]
pub fn command_injection(input: String) {
    trace!("[command_injection] Entering with input: {:?}", input);
    let (mut _rx, mut _child) = Command::new(input).spawn().expect("Failed command");
}
