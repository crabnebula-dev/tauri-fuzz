# Bibliography

## Fuzzer

- Generic
    - hongfuzz
    - Afl++
    - LibAFL
    - [Jackalope](https://github.com/googleprojectzero/Jackalope):
        - framework to easily build a black-box fuzzer
        - uses TinyInst
- Rust
    - cargo-fuzz: tool to invoke libfuzzer
    - libfuzzer: archived
    - cargodd
    - afl.rs: crate from AFL
    - cargo-libafl: wrapper around simple libafl fuzzer
    - fuzzcheck: not updated since a year
- Java serialization
    - Problems
        - Java serialization is flawed and input stream are converted to `Object`
        - Attacker can feed any kind of byte stream to be deserialized and can trigger
        gadget execution
        - This is much more difficult in Rust since target type for deserialization
        is defined at compile time
        - This could be done if the deserialization has intricate invariant checking
    - [ODDFuzz](https://arxiv.org/pdf/2304.04233.pdf)
        - ODD for _Open Dynamic deserialization_
        - uses lightweight taint analysis to identify potential gadget chains
        - new guided fuzzing towards sensitive code rather than coverage
- JS
    - [Fuzzilli](https://github.com/googleprojectzero/fuzzilli)
        - generates synctatically and semantically valid JS scripts for fuzzing
        - mutates over a custom intermediate language rather than source or AST
    - [JIT-Picking](https://publications.cispa.saarland/3773/1/2022-CCS-JIT-Fuzzing.pdf)
        - Differential Fuzzing of JavaScript Engines
        - differential fuzz JS engines with and without JIT optimizations
        - transparent probing of the state so it does not interfere with JIT optimizations
        - an execution hash depending on the observed variables values/types is calculated
        along the execution and sent to the fuzzer at the end for comparison
    - [Montage]()
        - neural network guided fuzzer
        - fuzz JS engines
- Windows
    - WinAFL: AFL-based fuzzer for Windows
- Fuzzer composition
    - Definition:
        - there are no generic best fuzzers
        - fuzzers perform differently depending on targets and resource usage
    - [autofz](https://github.com/sslab-gatech/autofz): compose a set of fuzzers to use depending
    on the target and fuzz "trend" at runtime
- Concurrency
    - [DDrace](https://www.usenix.org/conference/usenixsecurity23/presentation/yuan-ming):
    specialized in use-after free (UAF)
        - reduce search space by targeting potentially vulnerable block code
        - new metric to calculate "UAF distance"
- Web app fuzzing (in detail in section below)
- REST API fuzzing (maybe TODO)
    - Challenges
        - it's hard to trigger long sequence valid requests to trigger hard-to reach states
        - it's hard to forge high-quality requests that that pass the cloud service checking
    - [Miner](https://www.usenix.org/system/files/sec23fall-prepub-129-lyu.pdf): TODO
        - uses data history to guide fuzzing
        - uses AI attention model to produce _param-value_ list for each request
        - uses request response checker to keep interesting testcase
    - [RESTler]
- Trusted App (TA) using Trusted Execution Environment (TEE)
    - Challenge: this is harder than blackbox because the TEE prevents runtime analysis
    - you can only use inputs and outputs coming out from the TEE
    - [TEEzz](https://hexhive.epfl.ch/publications/files/23Oakland.pdf)
- Fuzzing during RTL development stage (specifications)
    - Advantage: fuzzing is done before production of the system therefore patching is less costly
    - [SpecDoctor](https://lifeasageek.github.io/papers/jaewon-specdoctor.pdf)
        - focuses against transient vulnerabilities
        - proposes a fuzzing template to emulate different scenarios
        (which part of the system is compromised)
        - uses differential fuzzing to identify side-channel behaviour
- Spec fuzzing
    - use fuzzing to test the completeness of a specification
    - [Fast](https://cs.uwaterloo.ca/~m285xu/assets/publication/fast-paper.pdf)
        - [Fast]() produces mutations on a program code we call CODE
        - the goal is
        - CODE mutants of the target program are both tested against
            - the original program test suite
            - against the [Move prover]()
                - the [Move prover]() takes both CODE and SPEC
                - CODE and SPEC will be compiled into Boogie
                - you can then uses an SMT solver to solve the Boogie input
        - results from both the test suite and the move prover can be compared
        to point out potential omission in the SPEC
   

### Web applications fuzzing

Fuzzing in web apps is still young.
- Blackbox
    - Pros/Cons
        - ++ you don't need source code
        - -- the inputs space is restrained in webapps and need manual meddling
        - -- vulnerabilities are inferred based on the output of the webapp which is not precise
    - In our case we got access to the source code? TODO
- Whitebox
    - no recent papers using this approach
    - Pros/Cons
        - -- requires source code
        - -- usually uses language model making them language-specific
        - ++ the fuzzing is the most complete
- Greybox
    - really few papers of this type but it looks promising

#### Challenges

What challenges are specific to web applications?

- webapps have many components that we don't want to fuzz
    - web server that takes HTTP request
    - data storage
    - most likely a code runtime
    - the app we want to test
- __Enabling fuzzing for webapps__
    - detecting inputs that triggers vulnerabilities
        - binary fuzzing usually detects segfault
    - generating valid inputs for end-to-end execution
        - inputs need to be valid HTTP requests
        - inputs need to possess the necessary input parameters for the webapp logic
- __Improving fuzzing for webapps__
    - collecting coverage information
        - not always possible with web applications
    - mutating inputs effectively
        - little research has been done on mutation strategy on webapps currently

#### Industry solution

- [Burp](https://portswigger.net/burp/documentation/desktop/tools/burps-browser)
- [Skipfish](https://portswigger.net/burp/documentation/desktop/tools/burps-browser)

#### [WebFuzz](https://www.researchgate.net/publication/354942205_webFuzz_Grey-Box_Fuzzing_for_Web_Applications)

Date: 2021
[Github](https://github.com/ovanr/webFuzz)

##### Contributions

- greybox fuzzer targeted at PHP web applications specialized for XSS vulnerabilities
- bug injection technique in PHP code
    - useful to evaluate webFuzz and other bug-finding techniques in webapps

##### Fuzzer
- uses edge coverage on PHP server code
- __workflow__
    - fuzzer fetches any GET or POST request that has been uncovered by a crawler
    - sends the request to the webapp
    - reads its HTTP response and coverage feedback
        - http is parsed to uncover new potential HTTP requests and XSS vulnerabilities
        - if feedback is favorable, store the HTTP request for further mutations
    - loop
- __HTTP requests mutation__
    - modify parameters of POST and GET request
    - 5 mutations techniques are employed
        - insertion of real XSS payloads
        - mixing GET or POST parameters from previously interesting requests
        - insertion of randomly generated strings
        - insertion of HTML, JS or PHP tokens
        - altering the type of a parameter
- __web crawling__
    - HTTP responses are parsed and analysed to crawl the whole app
    - extract new fuzz targets from `anchor` and `form` elements
    - retrieve inputs from `input`, `textarea` and `option` elements
- __vulnerability detection__
    - look for stored and reflective XSS vulnerabilities
        - stored XSS when JS is stored in the webapp data
        - reflective XSS vuln when JS from an HTTP request is reflected on the webapp
    - HTML responses are parsed and analysed to discover code in
        - link attribute (e.g. `href`) that start with the `javascrip:` label
        - executable attribute that starts with the `on` prefix (e.g. `onclick`)
        - script elements
    - fuzzer injects XSS payloads in the HTTP requests to call `alert()`
        - fuzzer detector check for any calls to `alert()`
- __corpus selection__ criteria
    - coverage score: number of labels triggered
    - mutated score: difference of code coverage with its parent request it
    was mutated from
    - sinks present: if the request managed to find their way in the HTTPS response
    - execution time: round-trip time of the request
    - size: number of char in the request
    - picked score: number of times it was picked for further mutations


#### [Witcher](https://adamdoupe.com/publications/witcher-oakland2023.pdf)

Date: 2023
Greybox fuzzing

##### Contributions

- framework to ease the integration of coverage-guided fuzzing on webapps
- fuzzer that can detect multiple type of vulnerabilities in both server-side binary
and interpreted web applications
    - SQL injection, command injection, memory corruption vulnerability (in C)

##### Enable fuzzing in webapp for SQL and command injection

###### Fault Escalator

   
#### [BlackWidow](https://www.cse.chalmers.se/~andrei/bw21.pdf)

Date: 2021
[Black Widow Github](https://github.com/1N3/BlackWidow)
TODO

#### [BackREST](https://arxiv.org/pdf/2108.08455.pdf)

Date: 2021
Blackbox fuzzing

## Bug oracles

## Program instrumentation

The goal is to instrument the fuzzed program to obtain metrics during fuzzing.
These metrics are either to guide mutation of inputs or detecting "dangerous" behaviour.
Programs needs to be instrumented to give this kind of info.
Instrumentation can be done at different levels:
- at source code
- during compilation, usually AST
- binary

### Bug oracles

Metrics that tells the fuzzer that it has detected a potential bug:
- segfaults and signals
- memory sanitizer
    - [Google sanitizers for LLVM](https://github.com/google/sanitizers)
    - ASAN, MSAN
- assertions in the code
- different behaviour in differential fuzzing
    - memory state
    - message passing

### Metrics to improve fuzzing

Metrics that are collected and use to improve the selection of future inputs:
- code coverage
- code targeting: how fast it is to access specific code
- "distance" to certain type of vulnerabilities
- logging for better understanding of the program
- [power consumption leaks](https://arxiv.org/abs/1908.05012)

#### Code coverage

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
       


### Tools for runtime instrumentation/tracing

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


### Binary instrumentation

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

### Compile-time instrumentation

Multiple advantages:
- speed: compiler can still optimize code after instrumentation
- portability: the instrumentation is architecture independent

Tools:
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
        - intensification optimize a promising area
            - focus on current best solution
            - mutate to find the better neighboring solutions
- Nautilus: combines usage of grammar + code coverage results
    - improve the probability of having syntactically and semantically valid inputs
- Grimoire

       
## Benchmarks

Fuzzers efficiency are compared over different metrics/conditions
Overall there is no best fuzzer.
Fuzzers performance vary a lot depending on the fuzzing environment:
- resources given
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

