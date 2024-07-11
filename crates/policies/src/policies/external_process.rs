use crate::engine::{FunctionPolicy, FuzzPolicy, Rule, RuleError};
#[cfg(not(target_env = "msvc"))]
pub use not_msvc::*;
use std::process::Command;
use std::sync::Arc;

/// These are the functions that the Rust `Command` API gives to start an external
/// binary in a new process
const MONITORED_FUNCTIONS_AT_ENTRY: [&str; 1] = [
    "std::process::Command::output",
    // "std::process::Command::status",
    // "std::process::Command::spawn",
];

/// These are the functions that the Rust `Command` API gives to get the return value
/// of an external binary that has been started
const MONITORED_FUNCTIONS_AT_EXIT: [&str; 5] = [
    "std::process::Command::output",
    "std::process::Command::status",
    "std::process::Child::wait",
    "std::process::Child::try_wait",
    "std::process::Child::wait_with_output",
];

#[cfg(not(target_env = "msvc"))]
mod not_msvc {
    use super::*;

    fn block_monitored_binaries_on_entry(
        blocked_binaries: &[String],
        registers: &[usize],
    ) -> Result<bool, RuleError> {
        // This is unsafe because we assume that registers at index 2 is a point to a Rust `Command`
        let binary = unsafe {
            let ptr = registers[1] as *const Command;
            ptr.as_ref()
                .unwrap()
                .get_program()
                .to_str()
                .expect("Unexpected character in binary")
        };

        let is_monitored_binary = blocked_binaries
            .iter()
            .any(|blocked_binary| blocked_binary.ends_with(binary));

        Ok(!is_monitored_binary)
    }

    pub fn block_on_entry(blocked_binaries: Vec<String>) -> FuzzPolicy {
        let current_bin = std::env::current_exe()
            .expect("Failed to get binary path")
            .to_string_lossy()
            .to_string();
        MONITORED_FUNCTIONS_AT_ENTRY
            .iter()
            .map(move |f| {
                let name: String = (*f).into();
                let description = format!(
                    "Invocation to external binaries is blocked: {:?}]",
                    blocked_binaries
                );

                let blocked_binaries_clone = blocked_binaries.clone();
                FunctionPolicy {
                    name,
                    lib: current_bin.clone(),
                    rule: Rule::OnEntry(Arc::new(move |registers| {
                        block_monitored_binaries_on_entry(&blocked_binaries_clone, registers)
                    })),
                    description,
                    nb_parameters: 2,
                    is_rust_function: true,
                }
            })
            .collect()
    }

    pub fn block_on_error_status() -> FuzzPolicy {
        todo!();
        let current_bin = std::env::current_exe()
            .expect("Failed to get binary path")
            .to_string_lossy()
            .to_string();
        MONITORED_FUNCTIONS_AT_EXIT
            .iter()
            .map(|f| {
                let name: String = (*f).into();
                let description = format!("Access to [{}] is blocked", f);

                FunctionPolicy {
                    name,
                    lib: current_bin.clone(),
                    rule: Rule::OnEntry(todo!()),
                    // rule: Rule::OnEntry(block_on_entry),
                    description,
                    nb_parameters: 2,
                    is_rust_function: true,
                }
            })
            .collect()
    }
}
