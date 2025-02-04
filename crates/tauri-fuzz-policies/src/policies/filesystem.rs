// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

#![allow(dead_code)]
pub use file_policy_impl::*;

#[cfg(not(target_env = "msvc"))]
mod file_policy_impl {
    use crate::engine::{FunctionPolicy, FuzzPolicy, Rule, RuleError};
    use crate::policies::{block_on_entry, LIBC};
    use std::sync::Arc;

    // Functions that are monitored when it comes to file system access
    const MONITORED_FUNCTIONS: [&str; 3] = ["fopen", "open", "open64"];

    pub fn no_file_access() -> FuzzPolicy {
        MONITORED_FUNCTIONS
            .iter()
            .map(|f| {
                let name: String = (*f).into();
                let description = format!("Access to [{f}] denied");
                FunctionPolicy {
                    name,
                    lib: LIBC.into(),
                    rule: Rule::OnEntry(block_on_entry()),
                    description,
                    nb_parameters: 2,
                    is_rust_function: false,
                }
            })
            .collect()
    }

    // Check if the flag is READ_ONLY
    fn block_non_read_only(params: &[usize]) -> Result<bool, RuleError> {
        let flag = params[1];
        let res = (flag & libc::O_ACCMODE as usize) == libc::O_RDONLY as usize;
        Ok(!res)
    }

    pub fn read_only_access() -> FuzzPolicy {
        MONITORED_FUNCTIONS
            .iter()
            .map(|f| {
                let name: String = (*f).into();
                let description = format!("Access to [{f}] is only allowed with read-only access");
                FunctionPolicy {
                    name,
                    lib: LIBC.into(),
                    rule: Rule::OnEntry(Arc::new(block_non_read_only)),
                    description,
                    nb_parameters: 2,
                    is_rust_function: false,
                }
            })
            .collect()
    }

    // Check if the flag is WRITE_ONLY
    fn block_non_write_only(params: &[usize]) -> Result<bool, RuleError> {
        let flag = params[1];
        let res = (flag & libc::O_ACCMODE as usize) == libc::O_WRONLY as usize;
        Ok(!res)
    }

    pub fn write_only_access() -> FuzzPolicy {
        MONITORED_FUNCTIONS
            .iter()
            .map(|f| {
                let name: String = (*f).into();
                let description = format!("Access to [{f}] is only allowed with write-only access");
                FunctionPolicy {
                    name,
                    lib: LIBC.into(),
                    rule: Rule::OnEntry(Arc::new(block_non_write_only)),
                    description,
                    nb_parameters: 2,
                    is_rust_function: false,
                }
            })
            .collect()
    }

    /// Checks if the filename contained in the first register is part of the blocked files
    fn block_access_to_filenames(
        blocked_files: &[String],
        registers: &[usize],
    ) -> Result<bool, RuleError> {
        let filename = unsafe { crate::policies::utils::nth_argument_as_str(registers, 0) };

        Ok(blocked_files
            .iter()
            .any(|blocked_filename| filename.ends_with(blocked_filename)))
    }

    /// Block access to file with name [`filename`].
    pub fn no_access_to_filenames(blocked_files: Vec<String>) -> FuzzPolicy {
        MONITORED_FUNCTIONS
            .iter()
            .map(move |f| {
                let name: String = (*f).into();
                let description = format!(
                    "Access to following files is denied: {:?}",
                    blocked_files.clone()
                );
                let blocked_files_clone = blocked_files.clone();
                FunctionPolicy {
                    name,
                    lib: LIBC.into(),
                    rule: Rule::OnEntry(Arc::new(move |registers| {
                        block_access_to_filenames(&blocked_files_clone, registers)
                    })),
                    description,
                    nb_parameters: 2,
                    is_rust_function: false,
                }
            })
            .collect()
    }
}

#[cfg(target_env = "msvc")]
mod file_policy_impl {
    use core::slice;

    use crate::engine::{FunctionPolicy, FuzzPolicy, Rule, RuleError};
    use crate::policies::block_on_entry;
    use nt_string::unicode_string::NtUnicodeStr;
    use std::sync::Arc;
    use windows_sys::Wdk::Foundation::OBJECT_ATTRIBUTES;
    use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};

    const FILE_CRT: &str = "ntdll.dll";
    // Doc to NtCreateFile: https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntcreatefile
    const OPEN_FILE: &str = "NtCreateFile";

    pub fn no_file_access() -> FuzzPolicy {
        vec![FunctionPolicy {
            name: OPEN_FILE.into(),
            lib: FILE_CRT.into(),
            rule: Rule::OnEntry(block_on_entry()),
            description: format!("Access to [{FILE_CRT}::{OPEN_FILE}] denied"),
            nb_parameters: 11,
            is_rust_function: false,
        }]
    }

    /// Policy where file access can only be read_only
    pub fn read_only_access() -> FuzzPolicy {
        vec![FunctionPolicy {
            name: OPEN_FILE.into(),
            lib: FILE_CRT.into(),
            rule: Rule::OnEntry(Arc::new(block_non_read_only_flag)),
            description: format!("Access to [{FILE_CRT}::{OPEN_FILE}] restricted to read-only"),
            nb_parameters: 11,
            is_rust_function: false,
        }]
    }

    const GENERIC_MASK: u32 = 0xf0000000;
    // Checks if the flag is READ_ONLY
    // NOTE: flag values seems to differ from the documentation:
    // https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntcreatefile
    // Refer to the diary for more details
    fn block_non_read_only_flag(params: &[usize]) -> Result<bool, RuleError> {
        let flag = params[1] as u32;
        let res = (flag & GENERIC_MASK) == GENERIC_READ;
        Ok(!res)
    }

    /// Policy where file access can only be write_only
    pub fn write_only_access() -> FuzzPolicy {
        vec![FunctionPolicy {
            name: OPEN_FILE.into(),
            lib: FILE_CRT.into(),
            rule: Rule::OnEntry(Arc::new(block_non_write_only_flag)),
            description: format!("Access to [{FILE_CRT}::{OPEN_FILE}] restricted to read-only"),
            nb_parameters: 11,
            is_rust_function: false,
        }]
    }

    // Checks if the flag is WRITE_ONLY
    // NOTE: flag values seems to differ from the documentation. Refer to the diary for more
    // details
    fn block_non_write_only_flag(params: &[usize]) -> Result<bool, RuleError> {
        let flag = params[1] as u32;
        let res = (flag & GENERIC_MASK) == GENERIC_WRITE;
        Ok(!res)
    }

    /// Checks if the filename contained in the first register is part of the blocked files
    fn block_access_to_filenames(
        blocked_files: &[String],
        registers: &[usize],
    ) -> Result<bool, RuleError> {
        let obj_attr_ptr = registers[2] as *const OBJECT_ATTRIBUTES;
        unsafe {
            let obj_attr: OBJECT_ATTRIBUTES = *obj_attr_ptr;

            // Convert win32 UNICODE_STRING to a rust String
            let filename_buffer = (*obj_attr.ObjectName).Buffer;
            let filename_slice =
                slice::from_raw_parts(filename_buffer, (*obj_attr.ObjectName).Length as usize);
            // Get a proper UNICODE_STRING parsing with the nt-string crate
            let unicode_data =
                NtUnicodeStr::try_from_u16_until_nul(filename_slice).map_err(|_| {
                    RuleError::ParametersTypeConversionError(String::from(
                        "Failed to get Unicode string from parameter",
                    ))
                })?;

            let file_path = String::from_utf16_lossy(unicode_data.as_slice());
            Ok(blocked_files
                .iter()
                .any(|blocked_filename| file_path.ends_with(blocked_filename)))
        }
    }

    /// Block access to file with name [`filename`].
    pub fn no_access_to_filenames(blocked_files: Vec<String>) -> FuzzPolicy {
        let blocked_files_clone = blocked_files.clone();
        vec![FunctionPolicy {
            name: OPEN_FILE.into(),
            lib: FILE_CRT.into(),
            rule: Rule::OnEntry(Arc::new(move |registers| {
                block_access_to_filenames(&blocked_files_clone, registers)
            })),
            description: format!("Access to files {:?} denied", blocked_files),
            nb_parameters: 11,
            is_rust_function: false,
        }]
    }
}
