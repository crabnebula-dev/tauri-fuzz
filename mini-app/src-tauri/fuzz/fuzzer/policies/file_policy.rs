#![allow(dead_code)]
use crate::policies::FunctionPolicy;
use crate::policies::Rule;
use crate::policies::LIBC;

pub fn no_file_access() -> Vec<FunctionPolicy> {
    vec![
        FunctionPolicy {
            name: "fopen".into(),
            lib: LIBC.into(),
            rule: Rule::OnEntry(),
            description: "Access to [fopen] denied".into(),
            nb_parameters: 2,
        },
        FunctionPolicy {
            name: "open".into(),
            lib: LIBC.into(),
            rule: Rule::OnEntry(),
            description: "Access to [open] denied".into(),
            nb_parameters: 2,
        },
        FunctionPolicy {
            name: "open64".into(),
            lib: LIBC.into(),
            rule: Rule::OnEntry(),
            description: "Access to [open64] denied".into(),
            nb_parameters: 2,
        },
    ]
}

// Taken from libc crate
// https://docs.rs/libc/latest/libc/constant.O_WRONLY.html
const READ_ONLY_FLAG: usize = 0;
const WRITE_ONLY_FLAG: usize = 1;
const READ_WRITE_FLAG: usize = 2;
const ACCESS_MODE_MASK: usize = 3;

// Check if the flag is READ_ONLY
fn no_write_flag(flag: usize) -> bool {
    let res = (flag & ACCESS_MODE_MASK) == READ_ONLY_FLAG;
    // println!("flag: {:?}", flag);
    // println!("res: {:?}", res);
    res
}

pub fn no_write_access() -> Vec<FunctionPolicy> {
    vec![
        FunctionPolicy {
            name: "fopen".into(),
            lib: LIBC.into(),
            rule: Rule::OnParameter(1, no_write_flag),
            description: "Access to [fopen] with write access denied".into(),
            nb_parameters: 2,
        },
        FunctionPolicy {
            name: "open".into(),
            lib: LIBC.into(),
            rule: Rule::OnParameter(1, no_write_flag),
            description: "Access to [open] with write access denied".into(),
            nb_parameters: 2,
        },
        FunctionPolicy {
            name: "open64".into(),
            lib: LIBC.into(),
            rule: Rule::OnParameter(1, no_write_flag),
            description: "Access to [open64] with write access denied".into(),
            nb_parameters: 2,
        },
    ]
}
