// Inspired from https://github.com/phip1611/direct-syscalls-linux-from-rust/blob/main/src/main.rs
#![allow(dead_code)]
// use crate::LinuxFileFlags::{O_APPEND, O_CREAT, O_RDONLY, O_WRONLY};
use std::arch::asm;

const STDOUT_FD: u64 = 1;

#[tauri::command]
pub fn write_to_stdout(s: &str) -> i64 {
    sys_write(STDOUT_FD, s.as_ptr(), s.len() as u64)
}

/// Wrapper around a Linux syscall with three arguments. It returns
/// the syscall result (or error code) that gets stored in rax.
unsafe fn syscall_3(num: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    let res;
    asm!(
        // there is no need to write "mov"-instructions, see below
        "syscall",
        // from 'in("rax")' the compiler will
        // generate corresponding 'mov'-instructions
        in("rax") num,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        lateout("rax") res,
    );
    res
}

/// Small subset of the available Linux syscalls.
#[repr(u64)]
enum LinuxSysCalls {
    Read = 0,
    Write = 1,
    Open = 2,
    WriteV = 20,
}

/// Opens a file. Works like `open` in C.
fn sys_open(path: *const u8, flags: u32, umode: u16) -> i64 {
    unsafe {
        syscall_3(
            LinuxSysCalls::Open as u64,
            path as u64,
            flags as u64,
            umode as u64,
        )
    }
}

/// Opens a file. Works like `open` in C.
fn sys_read(fd: u64, buf: *mut u8, size: u64) -> i64 {
    unsafe { syscall_3(LinuxSysCalls::Read as u64, fd, buf as u64, size as u64) }
}

/// Linux write system call. Works like `write()` in C.
fn sys_write(fd: u64, data: *const u8, len: u64) -> i64 {
    unsafe { syscall_3(LinuxSysCalls::Write as u64, fd, data as u64, len) }
}
