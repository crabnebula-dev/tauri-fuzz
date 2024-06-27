use crate::engine::{FuzzPolicy, RuleError};
pub mod execv;
pub mod filesystem;

#[cfg(unix)]
pub(crate) const LIBC: &str = "libc";

pub fn no_policy() -> FuzzPolicy {
    vec![]
}

pub(crate) fn block_on_entry(_: &[usize]) -> Result<bool, RuleError> {
    Ok(false)
}
