use crate::engine::{FunctionPolicy, FuzzPolicy, Rule};
use crate::policies::{block_on_entry, LIBC};
pub use no_execv_policy_impl::*;

#[cfg(not(target_env = "msvc"))]
mod no_execv_policy_impl {
    use super::*;

    // A function that will create our `FuzzPolicy` at runtime
    const MONITORED_FUNCTIONS: [&str; 1] = ["execve"];

    pub fn block_execv_on_error() -> FuzzPolicy {
        MONITORED_FUNCTIONS
            .iter()
            .map(|f| {
                let name: String = (*f).into();
                let description = format!("Access to [{}] is blocked", f);

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
}
