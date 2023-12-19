//! Enumeration of libc functions the fuzzer should analyze
//! Taken from subcategores of https://en.wikipedia.org/wiki/C_standard_library

pub(crate) const LIBC_BLOCKED_FUNCTIONS: &[&str] = &[
    // For testing
    #[cfg(unix)]
    "geteuid",
    "fopen",
    "open",
    "open64",
];
