// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use crate::engine::{FunctionPolicy, FuzzPolicy, Rule, RuleError};
#[cfg(target_env = "msvc")]
pub use msvc::*;
#[cfg(not(target_env = "msvc"))]
pub use not_msvc::*;
use std::sync::Arc;

#[cfg(not(target_env = "msvc"))]
mod not_msvc {
    use super::*;
    use crate::policies::LIBC;
    use std::process::Command;

    /// These are the functions that the Rust `Command` API gives to start an external
    /// binary in a new process
    const MONITORED_RUST_API_EXTERNAL_PROCESS: [&str; 3] = [
        "std::process::Command::output",
        "std::process::Command::status",
        "std::process::Command::spawn",
    ];

    /// These are the functions that the Rust `Command` API gives to get the return value
    /// of an external binary that has been started
    const MONITORED_RUST_API_ERROR_STATUS: [&str; 5] = [
        "std::process::Command::output",
        "std::process::Command::status",
        "std::process::Child::wait",
        "std::process::Child::try_wait",
        "std::process::Child::wait_with_output",
    ];

    fn block_monitored_binaries_on_entry(
        blocked_binaries: &[String],
        registers: &[usize],
    ) -> Result<bool, RuleError> {
        // This is unsafe because we assume that registers at index 2 is a pointer to a Rust `Command`
        let binary = unsafe {
            let ptr = registers[1] as *const Command;
            ptr.as_ref()
                .unwrap()
                .get_program()
                .to_str()
                .expect("Unexpected character in binary")
        };

        println!("binary found: {binary:?}");

        let is_monitored_binary = blocked_binaries
            .iter()
            .any(|blocked_binary| blocked_binary.ends_with(binary));

        Ok(is_monitored_binary)
    }

    /// Policy that blocks creation of child process that executes specified binaries
    /// Only blocks child processes that were created through Rust API `std::Command::process`
    pub fn block_on_entry(blocked_binaries: Vec<String>) -> FuzzPolicy {
        let current_bin = std::env::current_exe()
            .expect("Failed to get binary path")
            .to_string_lossy()
            .to_string();
        MONITORED_RUST_API_EXTERNAL_PROCESS
            .iter()
            .map(move |f| {
                let name: String = (*f).into();
                let description =
                    format!("Invocation to external binaries is blocked: {blocked_binaries:?}]");

                let blocked_binaries_clone = blocked_binaries.clone();
                FunctionPolicy {
                    name,
                    lib: current_bin.clone(),
                    rule: Rule::OnEntry(Arc::new(move |registers| {
                        block_monitored_binaries_on_entry(&blocked_binaries_clone, registers)
                    })),
                    description,
                    nb_parameters: 2,
                    is_rust_function: true,
                }
            })
            .collect()
    }

    // Block calls to external binaries with the Rust api command function when they return an error
    fn block_rust_api_return_error(
        function_name: &str,
        register: usize,
    ) -> Result<bool, RuleError> {
        match function_name {
            "std::process::Command::output" | "std::process::Child::wait_with_output" => {
                let output: &std::process::Output = unsafe {
                    let ptr = register as *const std::io::Result<std::process::Output>;
                    ptr.as_ref()
                        .unwrap()
                        .as_ref()
                        .expect("Failed to execute process")
                    // ptr.as_ref().unwrap().expect("Failed to execute process")
                };
                let status = output.status;
                Ok(!status.success())
            }
            "std::process::Command::status" | "std::process::Child::wait" => {
                let exit_status: &std::process::ExitStatus = unsafe {
                    let ptr = register as *const std::io::Result<std::process::ExitStatus>;
                    ptr.as_ref()
                        .unwrap()
                        .as_ref()
                        .expect("Failed to execute process")
                };
                Ok(!exit_status.success())
            }
            "std::process::Child::try_wait" => {
                let exit_status: &Option<std::process::ExitStatus> = unsafe {
                    let ptr = register as *const std::io::Result<Option<std::process::ExitStatus>>;
                    ptr.as_ref()
                        .unwrap()
                        .as_ref()
                        .expect("Failed to execute process")
                };
                match exit_status {
                    None => Ok(true),
                    Some(status) => Ok(!status.success()),
                }
            }
            _ => unreachable!("This function is not monitored"),
        }
    }

    /// NOTE: This is not working on Windows. Binaries are stripped of
    /// their symbols and Frida can't find functions of the std lib
    /// This is acceptable since we have policy `block_on_child_process_error_status`
    /// that works and emcompasses more cases
    pub fn block_on_rust_api_error_status() -> FuzzPolicy {
        let current_bin = std::env::current_exe()
            .expect("Failed to get binary path")
            .to_string_lossy()
            .to_string();
        MONITORED_RUST_API_ERROR_STATUS
            .iter()
            .map(|f| {
                let name: String = (*f).into();
                let description = format!("External binary {f} returned with error status");

                FunctionPolicy {
                    name: name.clone(),
                    lib: current_bin.clone(),
                    rule: Rule::OnExit(Arc::new(move |return_value| {
                        block_rust_api_return_error(&name, return_value)
                    })),
                    description,
                    nb_parameters: 2,
                    is_rust_function: true,
                }
            })
            .collect()
    }

    const MONITORED_LIBC_API_ERROR_STATUS: [&str; 3] = ["wait", "waitid", "waitpid"];

    /// We store the status pointer that was given as argument. This will be used to get the child
    /// process exit status
    fn get_status_pointer(
        function_name: &str,
        parameters: &[usize],
        storage: &mut Option<usize>,
    ) -> Result<bool, RuleError> {
        let status_ptr = match function_name {
            "wait" => parameters[0],
            "waitpid" => parameters[1],
            "waitid" => unimplemented!(),
            _ => unreachable!(),
        };
        *storage = Some(status_ptr);
        Ok(false)
    }

    /// wait status was stored in `storage`
    /// Get child process exit status from `storage` and check if it's an error
    fn block_libc_return_error(
        _function_name: &str,
        register: usize,
        storage: &mut Option<usize>,
    ) -> Result<bool, RuleError> {
        if isize::try_from(register).expect("isize was expected from libc documentation") == -1 {
            // There was an error while waiting for child process
            return Ok(true);
        }
        let status_ptr: *const i32 = match storage {
            Some(v) => *v as *const i32,
            None => {
                return Err(RuleError::ExpectedStorageEmpty(
                    "Status pointer was not stored for evaluation of error".to_string(),
                ))
            }
        };
        unsafe {
            let child_exit_status = libc::WEXITSTATUS(*status_ptr);
            Ok(child_exit_status != 0)
        }
    }

    /// Block calls to  `wait`, `waitid` and `waitpid` when they get an error status
    /// -1 should be returned in case of failure
    pub fn block_on_child_process_error_status() -> FuzzPolicy {
        MONITORED_LIBC_API_ERROR_STATUS
            .iter()
            .map(|f| {
                let name: String = (*f).into();
                let name2: String = name.clone();
                let description = format!("External binary {f} returned with error status");
                FunctionPolicy {
                    name: name.clone(),
                    lib: LIBC.into(),
                    rule: Rule::OnEntryAndExit(
                        Arc::new(move |parameters, storage| {
                            get_status_pointer(&name, parameters, storage)
                        }),
                        Arc::new(move |return_value, storage| {
                            block_libc_return_error(&name2, return_value, storage)
                        }),
                        None,
                    ),
                    description,
                    nb_parameters: 3,
                    is_rust_function: false,
                }
            })
            .collect()
    }
}

#[cfg(target_env = "msvc")]
mod msvc {
    use super::*;

    const KERNEL32: &str = "KERNEL32.DLL";
    /// Functions that we are monitoring coupled with the library in which they reside
    /// These functions are used to wait for child processes
    /// TODO: Maybe reimplement this but with same functions but from `ntdll.dll`
    const MONITORED_WAIT_FUNCTIONS: [(&str, &str); 1] = [("GetExitCodeProcess", KERNEL32)];

    /// Functions that we are monitoring coupled with the library in which they reside
    /// These functions are used to create a child process
    /// TODO: Maybe reimplement this but with same functions but from `ntdll.dll`
    const MONITORED_CREATE_PROCESS_FUNCTIONS: [(&str, &str); 2] =
        [("CreateProcessA", KERNEL32), ("CreateProcessW", KERNEL32)];

    /// Policy that will block the execution anytime a child process returns an error
    pub fn block_on_child_process_error_status() -> FuzzPolicy {
        MONITORED_WAIT_FUNCTIONS
            .into_iter()
            .map(|(f, lib)| FunctionPolicy {
                name: f.into(),
                lib: lib.into(),
                rule: Rule::OnEntryAndExit(
                    Arc::new(store_status_ptr),
                    Arc::new(is_return_value_an_error),
                    None,
                ),
                description: format!("[`{f}`] exit code from child process is non-0 "),
                nb_parameters: 2,
                is_rust_function: false,
            })
            .collect::<FuzzPolicy>()
    }

    // Store value of status pointer positioned as 2nd parameter
    // This will be used when exiting the function to check the exit status
    // NOTE: not sure on the guarantees that parameter order is kept in C programs
    // Use with caution
    fn store_status_ptr(
        parameters: &[usize],
        storage: &mut Option<usize>,
    ) -> Result<bool, RuleError> {
        *storage = Some(parameters[1]);
        Ok(false)
    }

    /// Check status pointer from `GetExitCodeProcess` <https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getexitcodeprocess>
    /// NOTE: Convention is that success should have exit status to 0
    /// Though there are no guarantees that programs respect this convention
    fn is_return_value_an_error(
        return_value: usize,
        storage: &mut Option<usize>,
    ) -> Result<bool, RuleError> {
        // `GetExitCodeProcess` failed we let the original program handle it
        if return_value == 0 {
            return Ok(false);
        }

        // `GetExitCodeProcess` status pointer is of type `LPDWORD`
        // Following the crate `winapi` this corresponds to u32 raw pointer
        // <https://docs.rs/winapi/latest/winapi/shared/minwindef/type.LPDWORD.html>
        let status_ptr: *const u32 = match storage {
            Some(v) => *v as *const u32,
            None => {
                return Err(RuleError::ExpectedStorageEmpty(
                    "Status pointer was not stored for evaluation of error".to_string(),
                ))
            }
        };

        unsafe {
            println!("status_ptr: {:?}", status_ptr);
            println!("*status_ptr: {:?}", *status_ptr);

            // Standard is that success should return 0
            Ok(*status_ptr != 0)
        }
    }

    /// Block child processes that are created through `MONITORED_CREATE_PROCESS_FUNCTIONS`
    /// that runs a binary specified in `blocked_binaries`
    pub fn block_on_entry(blocked_binaries: Vec<String>) -> FuzzPolicy {
        MONITORED_CREATE_PROCESS_FUNCTIONS
            .into_iter()
            .map(|(f, lib)| {
                let blocked_binaries_clone = blocked_binaries.clone();
                FunctionPolicy {
                    name: f.into(),
                    lib: lib.into(),
                    rule: Rule::OnEntry(Arc::new(move |registers| {
                        block_monitored_binaries_on_entry(f, &blocked_binaries_clone, registers)
                    })),
                    description: format!(
                        "[`{f}`] These executables are not allowed: {:?}",
                        blocked_binaries
                    ),
                    nb_parameters: 10,
                    is_rust_function: false,
                }
            })
            .collect::<FuzzPolicy>()
    }

    use std::os::windows::prelude::OsStringExt;
    fn block_monitored_binaries_on_entry(
        function_name: &str,
        blocked_binaries: &[String],
        registers: &[usize],
    ) -> Result<bool, RuleError> {
        match function_name {
            "CreateProcessA" => {
                // Related docs: <https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessa>

                // Full path to executable
                // Type is LPCSTR
                let command_path = registers[0] as *const i8;
                // Command as if it was executed in cmd.exe
                // Type is LPSTR
                let command_line = registers[1] as *const i8;

                let block_command_path =
                    contains_blocked_binaries(lpstr_to_string(command_path), blocked_binaries);
                let block_command_line =
                    contains_blocked_binaries(lpstr_to_string(command_line), blocked_binaries);

                Ok(block_command_path || block_command_line)
            }
            "CreateProcessW" => {
                // Related docs: <https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw>

                // Full path to executable
                // Type is LPCWSTR
                let command_path = registers[0] as *const u16;
                // Command as if it was executed in cmd.exe
                // Type is LPWSTR
                let command_line = registers[1] as *const u16;

                let block_command_path =
                    contains_blocked_binaries(lpwstr_to_string(command_path), blocked_binaries);
                let block_command_line =
                    contains_blocked_binaries(lpwstr_to_string(command_line), blocked_binaries);

                Ok(block_command_path || block_command_line)
            }
            _ => unimplemented!(),
        }
    }

    fn lpstr_to_string(windows_str: *const i8) -> Option<String> {
        if windows_str.is_null() {
            return None;
        }

        let c_str;
        // The functions we are monitoring are supposed to be called with valid null-terminated strings
        unsafe {
            c_str = std::ffi::CStr::from_ptr(windows_str);
        }
        let rust_string = c_str.to_string_lossy().to_string();
        println!("ANSI: {}", rust_string);
        Some(rust_string)
    }

    fn lpwstr_to_string(windows_str: *const u16) -> Option<String> {
        if windows_str.is_null() {
            return None;
        }
        let slice;
        // The functions we are monitoring are supposed to be called with valid null-terminated strings
        unsafe {
            let length = (0..).take_while(|&i| *windows_str.offset(i) != 0).count();
            slice = std::slice::from_raw_parts(windows_str, length);
        }
        let rust_string = std::ffi::OsString::from_wide(slice)
            .to_string_lossy()
            .into_owned();
        println!("unicode: {}", rust_string);
        Some(rust_string)
    }

    /// Check if 'command' calls one of the blocked binaries
    /// NOTE: The check only checks if the command contains one of the blocked binary
    /// This is a bit loose and may produce errors
    fn contains_blocked_binaries(command: Option<String>, blocked_binaries: &[String]) -> bool {
        match command {
            None => false,
            Some(command) => blocked_binaries
                .iter()
                .any(|blocked_binary| command.contains(blocked_binary)),
        }
    }

    /// NOTE: This is not working on Windows.
    /// This could work similarly to the non-windows versions but in Windows binaries are stripped of their symbols.
    /// Hence Frida can't find functions of the std lib and we can't monitor functions from `std::process::Command`.
    /// This is acceptable since we have policy `block_on_child_process_error_status`
    /// that works and emcompasses more cases
    pub fn block_on_rust_api_error_status() -> FuzzPolicy {
        unimplemented!()
    }
}
