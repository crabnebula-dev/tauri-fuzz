use tauri_fuzz_tools::policies::{DenyCondition, FunctionDenyRule};

#[cfg(unix)]
const LIBC: &str = "libc";
#[cfg(windows)]
const LIBC: &str = "msvcrt";

// TODO should be a constant for better perf
// Init the fuzzer policy containing the security policiy rules
pub(crate) unsafe fn get_fuzz_policy() -> Vec<FunctionDenyRule> {
    vec![
        FunctionDenyRule {
            name: "fopen".into(),
            lib: LIBC.into(),
            deny_conditions: vec![DenyCondition::OnEntry()],
        },
        FunctionDenyRule {
            name: "open".into(),
            lib: LIBC.into(),
            deny_conditions: vec![DenyCondition::OnEntry()],
        },
        FunctionDenyRule {
            name: "open64".into(),
            lib: LIBC.into(),
            deny_conditions: vec![DenyCondition::OnEntry()],
        },
    ]
}
