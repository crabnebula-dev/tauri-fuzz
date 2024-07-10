use std::ffi::CStr;

/// This is unsafe because we assume that the registers chosen contain a C string
pub(crate) unsafe fn nth_argument_as_str(registers: &[usize], index: usize) -> &str {
    // the first register should contain a pointer to the name of the file being accessed
    let name_ptr = registers[index] as *const i8;
    let c_str = CStr::from_ptr(name_ptr);
    c_str
        .to_str()
        .expect("Failed to convert register into string")
}
