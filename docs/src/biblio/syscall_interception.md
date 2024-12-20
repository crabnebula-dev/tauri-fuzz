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

## Tools used

- [capstone](https://docs.rs/capstone/latest/capstone/index.html#)
    - multi-platform, multi-architecture disassembly framework
    - frida is used for binary analysis and the rely on capstone to disassemble the
    instruction to be able to operate on them
- [frida](https://frida.re/docs/home/)
    - dynamic code instrumentation toolkit
    - allows you to inject snippet of code in native apps
    - [frida-core]
        - its role it to attach/hijack the target program to be able to interact with it
            - package `frida-gum` as a shared lib which is injected into the target program
            - then provide a two way communication to interact with `frida-gum` with your scripts
        - this is the common way to use frida
        - sometimes it's not possible to do so (jail iOS or Android)
            - in this situation you can use `frida-gadget`
    - [frida-gadget](https://frida.re/docs/gadget/)
        - shared library meant to be loaded by the target program
        - multiple way to load this lib
            - modify the source code
            - LD_PRELOAD
            - patching one of the target program library
        - it's started as soon as the dynamic linker call its constructor function
    - [frida-gum](https://github.com/frida/frida-gum)
        - instrumentation and introspection lib
        - C lib but provide a JS api to interact with it
        - 3 instrumentation core
            - [Stalker](https://frida.re/docs/stalker/)
                - code tracing enging
                - capture every function/code/instruction that is executed
            - [Interceptor](https://docs.rs/frida-gum/latest/frida_gum/interceptor/index.html#)
                - function hooking engine
            - [MemoryAccessMonitor](https://github.com/frida/frida-gum/blob/main/gum/gummemoryaccessmonitor.h)

## Panic in Rust

### `panic!`

1. `std::panic::panic_any(msg)`
2. if exists, panic hook is called
    - `std::panic::set_hook`
3. if panic hook returns, unwind the thread stack
4. if the registers are messed up
    - the unwinding fails and thread aborts
    - else, per frame the data is dropped
        - if a panic is hit while dropping data then thread aborts
5. some frames may be marked as "catching" the unwind
    - marked via `std::panic::catch_unwind()`
6. When the thread has finished unwinding
    - if it's main thread then `core::intrisics::abort`
    - else, thread is terminated and can be collected with `std::thread::join`

### panic is memory costly

To unwind the stack some debugging information is added to the stack
- debug information in DWARF format
- in embedded `panic = abort` is used

