use crate::engine::{FunctionPolicy, FuzzPolicy, Rule, RuleError};
#[cfg(not(target_env = "msvc"))]
pub use not_msvc::*;
use std::process::Command;
use std::sync::Arc;

/// These are the functions that the Rust `Command` API gives to start an external
/// binary in a new process
const MONITORED_FUNCTIONS_AT_ENTRY: [&str; 3] = [
    "std::process::Command::output",
    "std::process::Command::status",
    "std::process::Command::spawn",
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

    // Check error status when the we get a Rust `std::process::Output` as return value
    fn block_output_with_error(_register: usize) -> Result<bool, RuleError> {
        // // This is unsafe because we assume that register contain a pointer to
        // // `std::process::Output`
        // let exit_status = unsafe {
        //     let ptr = register as *const std::process::Output;
        //     println!("Output {:?}", ptr.as_ref().unwrap());
        //     ptr.as_ref().unwrap().status
        // };
        //
        // println!("Exit status {:?}", exit_status);
        // Ok(exit_status.success())

        todo!(); // We don't get anything from casting the *const Output
    }

    pub fn block_on_error_status() -> FuzzPolicy {
        let current_bin = std::env::current_exe()
            .expect("Failed to get binary path")
            .to_string_lossy()
            .to_string();
        MONITORED_FUNCTIONS_AT_EXIT
            .iter()
            .map(|f| {
                let name: String = (*f).into();
                let description = format!("External binary {} returned with error status", f);

                FunctionPolicy {
                    name,
                    lib: current_bin.clone(),
                    rule: Rule::OnLeave(Arc::new(block_output_with_error)),
                    description,
                    nb_parameters: 2,
                    is_rust_function: true,
                }
            })
            .collect()
    }
}
