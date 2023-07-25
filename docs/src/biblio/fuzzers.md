# Fuzzer

## Generic Fuzzers

- hongfuzz
- Afl++
- LibAFL
- [Jackalope](https://github.com/googleprojectzero/Jackalope):
    - framework to easily build a black-box fuzzer
    - uses TinyInst

## Rust Fuzzers

- cargo-fuzz: tool to invoke libfuzzer
- libfuzzer: archived
- cargodd
- afl.rs: crate from AFL
- cargo-libafl: wrapper around simple libafl fuzzer
- fuzzcheck: not updated since a year

## Java Serialization Fuzzers

### Problems

- Java serialization is flawed and input stream are converted to `Object`
- Attacker can feed any kind of byte stream to be deserialized and can trigger
gadget execution
- This is much more difficult in Rust since target type for deserialization
is defined at compile time
- This could be done if the deserialization has intricate invariant checking

### Tools

[ODDFuzz](https://arxiv.org/pdf/2304.04233.pdf)
- ODD for _Open Dynamic deserialization_
- uses lightweight taint analysis to identify potential gadget chains
- new guided fuzzing towards sensitive code rather than coverage

## Javascript Engine Fuzzers

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

## Fuzzer for Windows

- WinAFL: AFL-based fuzzer for Windows

## Fuzzer Composition

### Definition

- there are no generic best fuzzers
- fuzzers perform differently depending on targets and resource usage

### Tools

[autofz](https://github.com/sslab-gatech/autofz): compose a set of fuzzers to use depending
on the target and fuzz "trend" at runtime

## Concurrency

[DDrace](https://www.usenix.org/conference/usenixsecurity23/presentation/yuan-ming):
specialized in use-after free (UAF)
- reduce search space by targeting potentially vulnerable block code
- new metric to calculate "UAF distance"

## Webapp fuzzing 

in detail in next chapter

## Trusted Environment Fuzzing

- Trusted App (TA) using Trusted Execution Environment (TEE)
    - Challenge: this is harder than blackbox because the TEE prevents runtime analysis
    - you can only use inputs and outputs coming out from the TEE
    - [TEEzz](https://hexhive.epfl.ch/publications/files/23Oakland.pdf)

## Fuzzing during RTL development stage (specifications)

- Advantage: fuzzing is done before production of the system therefore patching is less costly
- [SpecDoctor](https://lifeasageek.github.io/papers/jaewon-specdoctor.pdf)
    - focuses against transient vulnerabilities
    - proposes a fuzzing template to emulate different scenarios
    (which part of the system is compromised)
    - uses differential fuzzing to identify side-channel behaviour

## Spec fuzzing

use fuzzing to test the completeness of a specification

### [Fast](https://cs.uwaterloo.ca/~m285xu/assets/publication/fast-paper.pdf)

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

## Resources 

- [LibAFL](https://github.com/AFLplusplus/LibAFL)
- [Fuzzers Like Lego (CCC Talk)](https://aflplus.plus/rC3_talk_2020.pdf)
- [Tauri Commands Documentation](https://docs.rs/tauri/latest/tauri/command/index.html)
- [LibAFL paper from 2022](https://www.s3.eurecom.fr/docs/ccs22_fioraldi.pdf)
- [Fuzzy 101](https://epi052.gitlab.io/notes-to-self/blog/2021-11-01-fuzzing-101-with-libafl/)
- [AFL++ doc](https://aflplus.plus/docs/faq/)
- [AFL++ modules explanation](https://github.com/AFLplusplus/AFLplusplus/blob/stable/instrumentation/README.llvm.md)

