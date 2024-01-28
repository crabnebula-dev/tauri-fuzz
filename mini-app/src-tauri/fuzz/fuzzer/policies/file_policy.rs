#![allow(dead_code)]
use crate::policies::{block_on_entry, LIBC};
use tauri_fuzz_tools::policies::FunctionPolicy;
use tauri_fuzz_tools::policies::Rule;
use tauri_fuzz_tools::policies::RuleError;

pub fn no_file_access() -> Vec<FunctionPolicy> {
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

pub fn read_only_access() -> Vec<FunctionPolicy> {
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
