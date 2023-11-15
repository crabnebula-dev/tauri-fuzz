# System Calls Interception

The goal is to provide our fuzzer a runtime that intercept any system calls.

## Difficulty

- Intercept only system calls that originates from the fuzzed program and not from the fuzzer
- Try to not impact performance too much

## System calls

### Syscalls on Linux

- in x86 asm: `int $0x80` or `syscall` with syscall number in eax/rax
- through the libc

### Types of system calls

1. Process control
    - create process: `fork`
    - terminate process
    - load, execute
    - get/set process attributes
    - wait for time/event, signal event
    - allocate and free memory
2. File management
3. Device management
4. Information maintenance
    - get
5. Communication
6. Protection
    - get/set file permissions


## Intercepting system calls

- using frida
    - track for `int 0x80` or `syscalls`
    - calls to `libc`
- tools to intercept syscalls
    - [syscall\_intercept](https://github.com/pmem/syscall_intercept/tree/2c8765fa292bc9c28a22624c528580d54658813d)
    - [extrasafe](https://github.com/boustrophedon/extrasafe)
    - [LD_PRELOAD](https://man7.org/linux/man-pages/man8/ld.so.8.html)


With LibAFL Frida intercept any instruction 

## [syscall\_intercept](https://github.com/pmem/syscall_intercept/tree/2c8765fa292bc9c28a22624c528580d54658813d)

- library to intercept and hook on system calls
- not maintained
- only on Linux

## [extrasafe](https://github.com/boustrophedon/extrasafe)

- wrapper around [seccomp](https://www.kernel.org/doc/html/latest/userspace-api/seccomp_filter.html)
- only Linux

## [LD_PRELOAD](https://man7.org/linux/man-pages/man8/ld.so.8.html)

- Load specified shared object instead of default one 
- Can be used to override libc 
- This is specific to Unix systems 
    - on Windows you may use DLL injection

