use tauri_fuzz_tools::policies::{FuzzPolicy, RuleError};
pub mod file_policy;

#[cfg(unix)]
pub(crate) const LIBC: &str = "libc";

pub fn no_policy() -> FuzzPolicy {
    vec![]
}

pub(crate) fn block_on_entry(_: &Vec<usize>) -> Result<bool, RuleError> {
    Ok(false)
}
