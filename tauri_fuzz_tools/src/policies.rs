//! Enumeration of libc functions the fuzzer should analyze
//! Taken from subcategores of https://en.wikipedia.org/wiki/C_standard_library

use regex::Regex;
use std::fmt::Debug;
use std::ops::Range;

/// Set of deny conditions applied to a function
#[derive(Debug)]
pub struct FunctionDenyRule {
    /// Function name
    pub name: String,

    /// Lib in which the function resides
    pub lib: String,

    /// Conditions that can deny the function
    /// Any condition that returns true will trigger a crash
    pub deny_conditions: Vec<DenyCondition>,
}

/// Conditions that can be enforced to deny a function
#[derive(Debug)]
pub enum DenyCondition {
    /// Function is denied on entry
    OnEntry(),

    /// Function is denied due to a parameter having a denied value
    /// Tuple first value corresponds to the parameter index
    OnParameter((u8, LibcTypeRange)),

    /// Function is denied due to its return value being denied
    OnReturnValue(LibcTypeRange),
}

/// The different libc types expressed in range
#[derive(Debug)]
pub enum LibcTypeRange {
    Int(Range<i32>),
    CharPtr(Regex),
}

pub type FuzzPolicy = Vec<FunctionDenyRule>;
