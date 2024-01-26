use tauri_fuzz_tools::policies::{FunctionPolicy, FuzzPolicy, Rule};
pub mod file_policy;

#[cfg(unix)]
pub(crate) const LIBC: &str = "libc";
#[cfg(windows)]
pub(crate) const LIBC: &str = "msvcrt";

pub fn no_policy() -> FuzzPolicy {
    vec![]
}
