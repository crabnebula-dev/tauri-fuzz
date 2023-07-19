# Bibliography

## Fuzzer 

- C 
    - hongfuzz
- Rust 
    - cargo-fuzz: tool to invoke libfuzzer
    - libfuzzer: archived 
    - cargodd
    - afl.rs: crate from AFL
    - cargo-libafl: wrapper around simple libafl fuzzer
    - fuzzcheck: not updated since a year
- JS 
    - Fuzzili
    - Jackalope: coverage-guided blackbox fuzzing
- Windows 
    - WinAFL: AFL-based fuzzer for Windows
- Fuzzer composition 
    - Definition:
        - there are no generic best fuzzers
        - fuzzers perform differently depending on targets and resource usage
    - [autofz](https://github.com/sslab-gatech/autofz): compose a set of fuzzers to use depending 
      on the target and fuzz "trend" at runtime
- Concurrency 
    - [DDrace](https://www.usenix.org/conference/usenixsecurity23/presentation/yuan-ming): specialized in use-after free (UAF)
        - reduce search space by targeting potentially vulnerable block code
        - new metric to calculate "UAF distance"
- REST API fuzzing
    - [Miner](https://www.usenix.org/system/files/sec23fall-prepub-129-lyu.pdf): TODO

## Program instrumentation 

The goal is to instrument the fuzzed program to obtain metrics at runtime.
These metrics are usually used to guide mutation or generation of inputs.
Examples: 
- code coverage 
- "distance" to certain type of vulnerabilities
- logging for better understanding of the program

### Runtime instrumentation/tracing

This is used for blackbox fuzzing where you don't have access to the source code.

#### Tools

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


### Binary instrumentation

Also for blackbox fuzzing. 
Instrumentation is done only once, having better performance than with runtime instrumentation.

#### Tools 

Mainly from [AFL++ documentation](https://aflplus.plus/docs/fuzzing_binary-only_targets/)
- Dyninst
    - instruments the target at load time
    - save the binary with instrumentations
- Retrowrite: x86 binaries, decompiles to ASM which can be instrumented
    with afl-gcc
- Zafl: x86 binaries, decompiles to ASM which can be instrumented
    with afl-gcc

### Compile-time instrumentation

Multiple advantages: 
- speed: compiler can still optimize code after instrumentation 
- portability: the instrumentation is architecture independent

#### Tools

- CmpLog: logs results of comparison operands 
- afl-gcc-fast/afl-g++-fast: use gcc plugins

## Input Generation / Mutation

Target programs usually only accept certain kinds of inputs.
The goal

The usual main concerns:
- interesting inputs: input that produce new behaviour during fuzzing
    - this is really hard because it is often target-dependent 
    - there can be dynamic optimizations of the mutations during the fuzzing using fuzzing metrics
- syntactically correct: input should be valid 
    - can be solved given a grammar
    - grammar can be inferred
- semantically correct: input has to make sense
    - usually mutation over already valid inputs

### Tools

- Syzkaller: current fuzzer used in Linux Kernel development 
    - requires a grammar for syscalls
- [FuzzNG](https://www.ndss-symposium.org/ndss-paper/no-grammar-no-problem-towards-fuzzing-the-linux-kernel-without-system-call-descriptions/): fuzzer for linux kernel that does not require a grammar
    - FuzzNG reshapes the input space without needing a grammar
    - at runtime FuzzNG aware of which file descriptors and pointers are accessed 
    - before function calls, FuzzND pauses this process to populate these locations
- [Darwin](https://www.ndss-symposium.org/wp-content/uploads/2023/02/ndss2023_s159_paper.pdf): optimization of the mutation at runtime
    - issues:
        - optimal mutations are target-dependent 
        - runtime algorithms to optimize mutations may have a performance cost 
            that outweighs the benefits
    - leverages a variant of _Evolution strategy_
        - mix between evolution and intensifications
        - evolution optimizes over a set of inputs 
            - discover promising areas and avoid local optima
            - Algo:
                - start with primary population of inputs
                - natural selection of the population by selecting best inputs over a metric
                - mutate these best inputs to obtain a new population
                - reiterate the selection until certain criteria is met
        - intensification optimize a promizing area
            - focus on current best solution
            - mutate to find the better neighboring solutions 
- Nautilus
- Grimoire

        
## Benchs

Fuzzers efficiency are compared over different metrics/conditions
Overall there is no best fuzzer. 
Fuzzers performance vary a lot depending on the fuzzing environment:
- ressources given 
- time give 
- type of vulnerabilities
- target program

### Metrics used

From [Unifuzz](https://www.usenix.org/system/files/sec21summer_li-yuwei.pdf) paper:
- quantity of unique bugs
- quality of bugs: rare bugs are worth more
- speed of finding bugs 
- stability of finding bugs: can a fuzzer find the same bug repeatedly?
- coverage
- overheard

### Tools 

- Magma: real-world vulnerabilities with their ground truth to verify genuine software defects
- [Unifuzz](https://www.usenix.org/system/files/sec21summer_li-yuwei.pdf):
    real-world vulnerabilities with their ground truth to verify genuine software defects
- LAVA-M: benchs of programs with synctatically injected memory errors
- Cyber Grand Challenge: benchs of binaries with synthetic bugs
- Fuzzer Test Suite (FTS): real-world programs and vulnerabilities
- FuzzBench: improvement of FTS with a frontend for easier integration

# Some Resources and References

- [LibAFL](https://github.com/AFLplusplus/LibAFL)
- [Fuzzers Like Lego (CCC Talk)](https://aflplus.plus/rC3_talk_2020.pdf)
- [Tauri Commands Documentation](https://docs.rs/tauri/latest/tauri/command/index.html)
- [LibAFL paper from 2022](https://www.s3.eurecom.fr/docs/ccs22_fioraldi.pdf)
- [Fuzzy 101](https://epi052.gitlab.io/notes-to-self/blog/2021-11-01-fuzzing-101-with-libafl/)
- [AFL++ doc](https://aflplus.plus/docs/faq/)
- [AFL++ modules explanation](https://github.com/AFLplusplus/AFLplusplus/blob/stable/instrumentation/README.llvm.md)

