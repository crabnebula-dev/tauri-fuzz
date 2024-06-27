#![allow(dead_code)]
pub use file_policy_impl::*;

// TODO make this a macro
const BLOCKED_FILENAMES: [&str; 1] = ["foo.txt"];

#[cfg(not(target_env = "msvc"))]
mod file_policy_impl {
    use super::BLOCKED_FILENAMES;
    use crate::engine::{FunctionPolicy, FuzzPolicy, Rule, RuleError};
    use crate::policies::{block_on_entry, LIBC};
    use std::ffi::CStr;

    // Functions that are monitored when it comes to file system access
    const MONITORED_FUNCTIONS: [&str; 3] = ["fopen", "open", "open64"];

    pub fn no_file_access() -> FuzzPolicy {
        MONITORED_FUNCTIONS
            .iter()
            .map(|f| {
                let name: String = (*f).into();
                let description = format!("Access to [{}] denied", f);
                FunctionPolicy {
                    name,
                    lib: LIBC.into(),
                    rule: Rule::OnEntry(block_on_entry),
                    description,
                    nb_parameters: 2,
                }
            })
            .collect()
    }

    // FLAGS value for the [open], [fopen] functions
    const READ_ONLY_FLAG: usize = 0;
    const WRITE_ONLY_FLAG: usize = 1;
    const READ_WRITE_FLAG: usize = 2;
    const ACCESS_MODE_MASK: usize = 3;

    // Check if the flag is READ_ONLY
    fn read_only_flag(params: &[usize]) -> Result<bool, RuleError> {
        let flag = params[1];
        let res = (flag & ACCESS_MODE_MASK) == READ_ONLY_FLAG;
        Ok(res)
    }

    pub fn read_only_access() -> FuzzPolicy {
        MONITORED_FUNCTIONS
            .iter()
            .map(|f| {
                let name: String = (*f).into();
                let description =
                    format!("Access to [{}] is only allowed with read-only access", f);
                FunctionPolicy {
                    name,
                    lib: LIBC.into(),
                    rule: Rule::OnEntry(read_only_flag),
                    description,
                    nb_parameters: 2,
                }
            })
            .collect()
    }

    // Check if the flag is WRITE_ONLY
    fn is_write_only_flag(params: &[usize]) -> Result<bool, RuleError> {
        let flag = params[1];
        let res = (flag & ACCESS_MODE_MASK) == WRITE_ONLY_FLAG;
        Ok(res)
    }

    pub fn write_only_access() -> FuzzPolicy {
        MONITORED_FUNCTIONS
            .iter()
            .map(|f| {
                let name: String = (*f).into();
                let description =
                    format!("Access to [{}] is only allowed with write-only access", f);
                FunctionPolicy {
                    name,
                    lib: LIBC.into(),
                    rule: Rule::OnEntry(is_write_only_flag),
                    description,
                    nb_parameters: 2,
                }
            })
            .collect()
    }

    /// Checks if the filename contained in the first register is part of the blocked files
    fn rule_no_access_to_filenames(registers: &[usize]) -> Result<bool, RuleError> {
        let filename: &str;
        // Pointers in registers are supposed to be valid since they're sent from frida_gum
        unsafe {
            // the first register should contain a pointer to the name of the file being accessed
            let name_ptr = registers[0] as *const i8;
            let c_str = CStr::from_ptr(name_ptr);
            filename = c_str.to_str()?;
        }

        Ok(!BLOCKED_FILENAMES
            .iter()
            .any(|blocked_filename| filename.ends_with(blocked_filename)))
    }

    /// Block access to file with name [`filename`].
    pub fn no_access_to_filenames() -> FuzzPolicy {
        MONITORED_FUNCTIONS
            .iter()
            .map(|f| {
                let name: String = (*f).into();
                let description = format!(
                    "Access to {} denied. Blocked files are {:?}",
                    f, BLOCKED_FILENAMES
                );
                FunctionPolicy {
                    name,
                    lib: LIBC.into(),
                    rule: Rule::OnEntry(rule_no_access_to_filenames),
                    description,
                    nb_parameters: 2,
                }
            })
            .collect()
    }
}

#[cfg(target_env = "msvc")]
mod file_policy_impl {
    use core::slice;

    use super::BLOCKED_FILENAMES;
    use crate::engine::{FunctionPolicy, FuzzPolicy, Rule, RuleError};
    use crate::policies::block_on_entry;
    use nt_string::unicode_string::NtUnicodeStr;
    use windows_sys::Wdk::Foundation::OBJECT_ATTRIBUTES;
    use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};

    const FILE_CRT: &str = "ntdll.dll";
    // Doc to NtCreateFile: https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntcreatefile
    const OPEN_FILE: &str = "NtCreateFile";

    pub fn no_file_access() -> FuzzPolicy {
        vec![FunctionPolicy {
            name: OPEN_FILE.into(),
            lib: FILE_CRT.into(),
            rule: Rule::OnEntry(block_on_entry),
            description: format!("Access to [{FILE_CRT}::{OPEN_FILE}] denied"),
            nb_parameters: 11,
        }]
    }

    /// Policy where file access can only be read_only
    pub fn read_only_access() -> FuzzPolicy {
        vec![FunctionPolicy {
            name: OPEN_FILE.into(),
            lib: FILE_CRT.into(),
            rule: Rule::OnEntry(is_read_only_flag),
            description: format!("Access to [{FILE_CRT}::{OPEN_FILE}] restricted to read-only"),
            nb_parameters: 11,
        }]
    }

    const GENERIC_MASK: u32 = 0xf0000000;
    // Checks if the flag is READ_ONLY
    // NOTE: flag values seems to differ from the documentation:
    // https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntcreatefile
    // Refer to the diary for more details
    fn is_read_only_flag(params: &[usize]) -> Result<bool, RuleError> {
        let flag = params[1] as u32;
        let res = (flag & GENERIC_MASK) == GENERIC_READ;
        Ok(res)
    }

    /// Policy where file access can only be write_only
    pub fn write_only_access() -> FuzzPolicy {
        vec![FunctionPolicy {
            name: OPEN_FILE.into(),
            lib: FILE_CRT.into(),
            rule: Rule::OnEntry(is_write_only_flag),
            description: format!("Access to [{FILE_CRT}::{OPEN_FILE}] restricted to read-only"),
            nb_parameters: 11,
        }]
    }

    // Checks if the flag is WRITE_ONLY
    // NOTE: flag values seems to differ from the documentation. Refer to the diary for more
    // details
    fn is_write_only_flag(params: &[usize]) -> Result<bool, RuleError> {
        let flag = params[1] as u32;
        let res = (flag & GENERIC_MASK) == GENERIC_WRITE;
        Ok(res)
    }

    /// Checks if the filename contained in the first register is part of the blocked files
    fn rule_no_access_to_filenames(registers: &[usize]) -> Result<bool, RuleError> {
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
            Ok(!BLOCKED_FILENAMES
                .iter()
                .any(|blocked_filename| file_path.ends_with(blocked_filename)))
        }
    }

    /// Block access to file with name [`filename`].
    pub fn no_access_to_filenames() -> FuzzPolicy {
        vec![FunctionPolicy {
            name: OPEN_FILE.into(),
            lib: FILE_CRT.into(),
            rule: Rule::OnEntry(rule_no_access_to_filenames),
            description: format!("Access to files {:?} denied", BLOCKED_FILENAMES),
            nb_parameters: 11,
        }]
    }
}