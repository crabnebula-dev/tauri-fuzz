#![allow(dead_code)]
pub use file_policy_impl::*;

#[cfg(not(target_env = "msvc"))]
mod file_policy_impl {
    use crate::policies::{block_on_entry, LIBC};
    use std::ffi::CStr;
    use tauri_fuzz_tools::policies::{FunctionPolicy, FuzzPolicy, Rule, RuleError};

    pub fn no_file_access() -> FuzzPolicy {
        vec![
            FunctionPolicy {
                name: "fopen".into(),
                lib: LIBC.into(),
                rule: Rule::OnEntry(block_on_entry),
                description: "Access to [fopen] denied".into(),
                nb_parameters: 2,
            },
            FunctionPolicy {
                name: "open".into(),
                lib: LIBC.into(),
                rule: Rule::OnEntry(block_on_entry),
                description: "Access to [open] denied".into(),
                nb_parameters: 2,
            },
            FunctionPolicy {
                name: "open64".into(),
                lib: LIBC.into(),
                rule: Rule::OnEntry(block_on_entry),
                description: "Access to [open64] denied".into(),
                nb_parameters: 2,
            },
        ]
    }

    // FLAGS value for the [open], [fopen] functions
    const READ_ONLY_FLAG: usize = 0;
    const WRITE_ONLY_FLAG: usize = 1;
    const READ_WRITE_FLAG: usize = 2;
    const ACCESS_MODE_MASK: usize = 3;

    // Check if the flag is READ_ONLY
    fn read_only_flag(params: &Vec<usize>) -> Result<bool, RuleError> {
        let flag = params[1];
        let res = (flag & ACCESS_MODE_MASK) == READ_ONLY_FLAG;
        Ok(res)
    }

    pub fn read_only_access() -> FuzzPolicy {
        vec![
            FunctionPolicy {
                name: "fopen".into(),
                lib: LIBC.into(),
                rule: Rule::OnEntry(read_only_flag),
                description: "Access to [fopen] with write access denied".into(),
                nb_parameters: 2,
            },
            FunctionPolicy {
                name: "open".into(),
                lib: LIBC.into(),
                rule: Rule::OnEntry(read_only_flag),
                description: "Access to [open] with write access denied".into(),
                nb_parameters: 2,
            },
            FunctionPolicy {
                name: "open64".into(),
                lib: LIBC.into(),
                rule: Rule::OnEntry(read_only_flag),
                description: "Access to [open64] with write access denied".into(),
                nb_parameters: 2,
            },
        ]
    }
}

#[cfg(target_env = "msvc")]
mod file_policy_impl {
    use crate::policies::block_on_entry;
    use std::ffi::CStr;
    use tauri_fuzz_tools::policies::{FunctionPolicy, FuzzPolicy, Rule, RuleError};
    use windows_sys::Win32::Foundation::GENERIC_READ;

    const FILE_CRT: &str = "KERNEL32";
    const OPEN_FILE: &str = "CreateFileW";

    pub fn no_file_access() -> FuzzPolicy {
        // Doc to CreateFileW: https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew
        vec![FunctionPolicy {
            name: OPEN_FILE.into(),
            lib: FILE_CRT.into(),
            rule: Rule::OnEntry(block_on_entry),
            description: "Access to [CreateFileW] denied".into(),
            nb_parameters: 7,
        }]
    }

    // Check if the flag is READ_ONLY
    fn read_only_flag(params: &Vec<usize>) -> Result<bool, RuleError> {
        let flag = params[1];
        let res = flag == GENERIC_READ as usize;
        Ok(res)
    }

    pub fn read_only_access() -> FuzzPolicy {
        // Doc to CreateFileW: https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew
        vec![FunctionPolicy {
            name: OPEN_FILE.into(),
            lib: FILE_CRT.into(),
            rule: Rule::OnEntry(read_only_flag),
            description: "Access to [CreateFileW] restricted to read-only".into(),
            nb_parameters: 7,
        }]
    }

    // TODO make this a macro
    const BLOCKED_FILENAMES: [&'static str; 1] = ["foo.txt"];

    fn rule_no_access_to_filenames(registers: &Vec<usize>) -> Result<bool, RuleError> {
        let mut filename = "toto";
        println!("registers count: {}", registers.len());
        for reg in registers.iter() {
            unsafe {
                // the first register should contain a pointer to the name of the file being accessed
                let name_ptr = *reg as *const i8;
                let c_str = CStr::from_ptr(name_ptr);
                filename = c_str.to_str()?;
                println!("reg: {}", filename);
            }
        }
        // unsafe {
        //             // the first register should contain a pointer to the name of the file being accessed
        //             let name_ptr = registers[0] as *const i8;
        //             let c_str = CStr::from_ptr(name_ptr);
        //             filename = c_str.to_str()?;
        //         }

        // println!("jsfdkljfds: {}", filename);
        Ok(!BLOCKED_FILENAMES
            .iter()
            .any(|blocked_filename| filename.ends_with(blocked_filename)))
    }

    /// Block access to file with name [`filename`].
    pub fn no_access_to_filenames() -> FuzzPolicy {
        vec![FunctionPolicy {
            name: OPEN_FILE.into(),
            lib: FILE_CRT.into(),
            rule: Rule::OnEntry(rule_no_access_to_filenames),
            description: format!("Access to files {:?} denied", BLOCKED_FILENAMES),
            nb_parameters: 2,
        }]
    }
}
