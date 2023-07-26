# Program instrumentation

The goal is to instrument the fuzzed program to obtain metrics during fuzzing.
These metrics are either to guide mutation of inputs or detecting "dangerous" behaviour.
Programs needs to be instrumented to give this kind of info.
Instrumentation can be done at different levels:
- at source code
- during compilation, usually AST
- binary

## Bug oracles

Metrics that tells the fuzzer that it has detected a potential bug:
- segfaults and signals
- memory sanitizer
    - [Google sanitizers for LLVM](https://github.com/google/sanitizers)
    - ASAN, MSAN
- assertions in the code
- different behaviour in differential fuzzing
    - memory state
    - message passing

## Metrics to improve fuzzing

Metrics that are collected and use to improve the selection of future inputs:
- code coverage
- code targeting: how fast it is to access specific code
- "distance" to certain type of vulnerabilities
- logging for better understanding of the program
- [power consumption leaks](https://arxiv.org/abs/1908.05012)

### Code coverage

Multiple possible granularities:
- Basic block
    - __def__: maximal sequence of consecutive statements that are always
    executed together
    - measure which basic block get executed
    - this provides least granularity since the coverage does not cover
    basic block order of execution
- Branch/Edge coverage
    - measure the pair of consecutive blocks executed
    - a pair of basic block is called an _edge_
    - more precise and try to execute all conditional branches
    - __algo__:
        1. Give a unique label to all basic block
        2. Store any data related edge coverage to a global var
        3. At the beginning of each label xor current label and previous one
        4. This value is the edge label and is used as index for map coverage
        5. At the end of a basic block rightshift current label
            - this is to prevent 0-value label if basic block jumps on itself
        6. Store the rightshifted value as "previous visited block"
        7. At the end of program exec, print/send/store map coverage feedback

## Tools for runtime instrumentation/tracing

This is used for blackbox fuzzing where you don't have access to the source code.

Mainly from [afl++ doc](https://aflplus.plus/docs/fuzzing_binary-only_targets/):
- Frida: dynamic code instrumentation toolkit
    - you can inject JS script into your native apps
    - debug and script live process
    - usable on many platforms: Windows, Mac, Linux, iOS, Android, QNX
- Qemu: dynamic code injection using hooks
    - emulator
- TinyInst: runtime dynamic instrumentation library
    - more lightweight than other tools
    - easier to use but does not fit every usecase
    - MacOS and Windows only
- Nyx: only on Linux
- Unicorn: fork of Qemu
- Wine + Qemu: to run Win32 binaries
- Unicorn: fork of Qemu
- Tracing at runtime
    - Pintool: Intel x32/x64 on Linux, MacOS and Windows
    - Dynamorio
        - Intel x32/x64 on Linux, MacOS and Windows
        - Arm and AArch64
        - faster than Pintool but still slow
    - Intel-PT
        - use intels processor trace
        - downsides: buffer is small and debug info is complex
        - two AFL implementations: afl-pt and ptfuzzer
    - Coresight: ARM processor trace


## Binary instrumentation

Also for blackbox fuzzing.
Instrumentation is done only once, having better performance than with runtime instrumentation.

Mainly from [AFL++ documentation](https://aflplus.plus/docs/fuzzing_binary-only_targets/)
- Dyninst
    - instruments the target at load time
    - save the binary with instrumentations
- Retrowrite: x86 binaries, decompiles to ASM which can be instrumented
    with afl-gcc
- Zafl: x86 binaries, decompiles to ASM which can be instrumented
    with afl-gcc

## Compile-time instrumentation

Multiple advantages:
- speed: compiler can still optimize code after instrumentation
- portability: the instrumentation is architecture independent

Tools:
- CmpLog: logs results of comparison operands
- afl-gcc-fast/afl-g++-fast: use gcc plugins

### AFL tools

Recommendation from [AFL Guide to Fuzzing in Depth](https://github.com/AFLplusplus/AFLplusplus/blob/stable/docs/fuzzing_in_depth.md)
- `LTO > LLVM > gcc plugin > gcc mode`
- __Important__ checking the coverage of the fuzzing
    - use `afl-showmap`
    - Section 3.g of
    [AFL Guide to Fuzzing in Depth](https://github.com/AFLplusplus/AFLplusplus/blob/stable/docs/fuzzing_in_depth.md)

#### [LTO mode](https://github.com/AFLplusplus/AFLplusplus/blob/stable/instrumentation/README.lto.md)

- called `afl-clang-lto/afl-clang-lto++`
- works with llvm11 or newer
- __instrumentation at link time__
- autodictionary feature
    - while compiling a dictionary is generated
    - in fuzzing a dictionary are base inputs that will help improve code coverage
- improve efficiency of the fuzzer by avoiding basic block label collision
    - classic coverage labels blocks randomly
    - lto-mode instrument the files and avoid block label collision
- only con is that is has long compile time

#### LLVM mode

#### [gcc plugin mode](https://github.com/AFLplusplus/AFLplusplus/blob/stable/instrumentation/README.gcc_plugin.md)

Called `afl-gcc-fast/afl-g++-fast`
Instrument the target with the help of `gcc` plugins.

#### gcc/clang mode

The base version without any special features.
- `afl-gcc/afl-g++` and `afl-clang/afl-clang++`


