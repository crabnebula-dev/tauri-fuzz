//! Enumeration of libc functions the fuzzer should analyze
//! Taken from subcategores of https://en.wikipedia.org/wiki/C_standard_library

pub(crate) const LIBC_BLOCKED_FUNCTIONS: &[&str] = &[
    // For testing
    "geteuid", // File input/output https://en.wikipedia.org/wiki/C_file_input/output
    "fopen",
];
