use crate::engine::{ConditionOnParameters, FuzzPolicy};
pub mod external_process;
pub mod filesystem;
mod utils;

#[cfg(unix)]
pub(crate) const LIBC: &str = "libc.";

pub fn no_policy() -> FuzzPolicy {
    vec![]
}

pub(crate) fn block_on_entry() -> ConditionOnParameters {
    std::sync::Arc::new(|_| Ok(false))
}
