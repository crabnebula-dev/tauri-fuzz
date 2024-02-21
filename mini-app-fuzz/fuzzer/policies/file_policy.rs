#![allow(dead_code)]
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

// TODO make this a macro
const BLOCKED_FILENAMES: [&'static str; 1] = ["foo.txt"];
fn rule_no_access_to_filenames(registers: &Vec<usize>) -> Result<bool, RuleError> {
    let filename;
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
    vec![
        FunctionPolicy {
            name: "fopen".into(),
            lib: LIBC.into(),
            rule: Rule::OnEntry(rule_no_access_to_filenames),
            description: format!("Access to files {:?} denied", BLOCKED_FILENAMES),
            nb_parameters: 2,
        },
        FunctionPolicy {
            name: "open".into(),
            lib: LIBC.into(),
            rule: Rule::OnEntry(rule_no_access_to_filenames),
            description: format!("Access to files {:?} denied", BLOCKED_FILENAMES),
            nb_parameters: 2,
        },
        FunctionPolicy {
            name: "open64".into(),
            lib: LIBC.into(),
            rule: Rule::OnEntry(rule_no_access_to_filenames),
            description: format!("Access to files {:?} denied", BLOCKED_FILENAMES),
            nb_parameters: 2,
        },
    ]
}
