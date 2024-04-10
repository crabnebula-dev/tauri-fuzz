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

### Rust options 

Two code coverage options: 
- a GCC-compatible, gcov-based coverage implementation, enabled with `-Z profile`,
which derives coverage data based on DebugInfo
- a source-based code coverage implementation, enabled with `-C instrument-coverage`, 
which uses LLVM's native, efficient coverage instrumentation to generate very precise coverage data

#### [Rust Source-based coverage](https://doc.rust-lang.org/rustc/instrument-coverage.html)

- `cargo-fuzz` uses this technique
    - `cargo-fuzz` is not a fuzzer but a framework to call a fuzzer
    - the only supported fuzzer is `libFuzzer` 
    - through the `libfuzzer-sys` crate
- done on MIR
- based on llvm source-based code coverage
- `rustc -C instrument-coverage` does:
    - insert `llvm.instrprof.increment` at control-flows
    - add a map in each library and binary to keep track of coverage information
    - use symbol mangling v0
- uses the Rust profiler runtime
    - enabled by default on the `+nightly` channel
- needs to use a Rust demangler: [`rustfilt`](https://crates.io/crates/rustfilt)
    - can be provided to llvm options

##### Using it

1. Compile with `cargo` 
    - `RUSTFLAGS="-C instrument-coverage" cargo build`
    - may be necessary to use the profiler runtime:
    `RUSTC=$HOME/rust/build/x86_64-unknown-linux-gnu/stage1/bin/rustc`
2. Run the binary compiled
    - it should produce a file `default_*.profraw`
    - or name it with `LLVM_PROFILE_FILE="toto.profraw"`
3. Process coverage data with `llvm-profdata`
    - can be installed with `rustup`
    - `llvm-profdata merge -sparse toto.profraw -o toto.profdata`
4. Create reports with `llvm-cov`
    - can be installed with `rustup`
    - create a report when combining _profdata_ with the binary
    - `llvm-cov show -Xdemangler=rustfilt target/debug/examples/toto \
    -instr-profile=toto.profdata \
    -show-line-counts-or-regions \
    -show-instantiations \
    -name=add_quoted_string`

### LLVM options 

LLVM has multiple options to instrument program during compilation
- Source Based Coverage 
- Sanitizer Coverage 
- `gcov`: A GCC-compatible coverage implementation which operates on DebugInfo. This is enabled by `-ftest-coverage` or `--coverage`

#### [Source-Based Coverage](https://clang.llvm.org/docs/SourceBasedCodeCoverage.html)

Operates on AST and preprocessor information directly
- better to map lines of Rust source code to coverage reports
- `-fprofile-instr-generate -fcoverage-mapping`

#### [Sanitizer Coverage](https://clang.llvm.org/docs/SanitizerCoverage.html#introduction)

- operates on LLVM IR
- `-fsanitize-coverage=trace-pc-guard` to trace with guards/closures
    - will insert a call to `__sanitizer_cov_trace_pc_guard(&guard_variable)` on every edge
    - `__sanitizer_cov_trace_pc_guard(&guard_variable)` can be
        - implemented by user
        - defaulted to a counter with `-fsanitize-coverage=inline-8bit-counters` 
        - defualted to a boolean flag with `-fsanitize-coverage=inline-bool-flag `
- partial instrumentation with `-fsanitize-coverage-allowlist=allowlist.txt` 
and `-fsanitize-coverage-ignorelist=blocklist.txt`
    - these lists are filled with function names

### LibAFL tools

LibAFL project has directories such as:
- `libafl_targets` that can be used for instrumentation
- `libafl_cc` a library that provide facilities to wrap compilers

#### [`cargo-libafl`](https://github.com/AFLplusplus/cargo-libafl/tree/main)

This is a replacement to `cargo-fuzz` which went into maintenance.
`cargo-libafl` is just a framework to prepare fuzzing.
The actual fuzzer is `libfuzzer-sys` that is maintained in `libafl_targets`.

1. `cargo libafl init`
    - create a directory `fuzz_targets`
2. `cargo libafl run <fuzz target name>`
    - `exec_build` gives
    `RUSTFLAGS="-Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-level=4 -Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-pc-table -L /home/adang/.local/share/cargo-libafl/rustc-1.70.0-90c5418/cargo-libafl-0.1.8/cargo-libafl -lcargo_libafl_runtime -Cllvm-args=-sanitizer-coverage-trace-compares --cfg fuzzing -Clink-dead-code -Cllvm-args=-sanitizer-coverage-stack-depth -Cdebug-assertions -C codegen-units=1" "cargo" "build" "--manifest-path" "/home/adang/boum/fuzzy/playground/rust-url/fuzz/Cargo.toml" "--target" "x86_64-unknown-linux-gnu" "--release" "--bin" "fuzz_target_1"`
    - 


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

### AFL options

#### [CmpLog](https://github.com/AFLplusplus/AFLplusplus/blob/stable/instrumentation/README.cmplog.md)

Log comparison operands in shared memory for the different mutators.
Many comparison operands check whether part of the state are equal to a preset byte range. 
These byte ranges are called magic bytes.

`CmpLog` guide mutations by linking the inputs to the states magic bytes.

#### [laf-intel](https://github.com/AFLplusplus/AFLplusplus/blob/stable/instrumentation/README.laf-intel.md)

"De-optimize" certain LLVM passes to increase code coverage.

#### [persistent-mode](https://github.com/AFLplusplus/AFLplusplus/blob/stable/instrumentation/README.persistent_mode.md)

Fuzz a program in a single forked process instead of forking a new process for each fuzz execution.
Really important if we fuzz with fork server, speed improvements x10-x20

#### [partial instrumentation](https://github.com/AFLplusplus/AFLplusplus/blob/stable/instrumentation/README.instrument_list.md)

Select which part of the code you want to instrument
