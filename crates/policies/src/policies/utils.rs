use std::ffi::CStr;

/// This is unsafe because we assume that the registers chosen contain a C string
pub(crate) unsafe fn nth_argument_as_str(registers: &[usize], index: usize) -> &str {
    let ptr = registers[index] as *const i8;
    let c_str = CStr::from_ptr(ptr);
    c_str
        .to_str()
        .expect("Failed to convert register into string")
}
