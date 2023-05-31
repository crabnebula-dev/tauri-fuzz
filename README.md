# Fuzzy prototype

Fuzzer prototype to use for Tauri applications

## Architecure

- `mini-app` a minimal Tauri application which is the fuzz target
- `fuzzer` directory containing the custom fuzzer

## State of the Art Fuzzing

- Architecture of a Fuzzer
- Types of fuzzers
  - Black/Grey/White box
  - Mutation/Generation based
  - Generalized/Specialized
- Popular fuzzers: AFL, libFuzzer, hongfuzz
- LibAFL framework
- Areas of research/improvement
  - roadblock bypassing
  - structure aware fuzzing
  - corpus scheduling
  - energy assignment

## Tauri Fuzzing

The `Dockerfile` is meant to run the fuzzer. The idea is to use Vscode DevContainers for
the fuzzer, as libAFL has some issues on some distros.

`docker build . -t test-fuzz`

`docker run -it --privileged test-fuzz`

`cd fuzzer`

`cargo build --release`

OR

Use the devcontainer feature from vscode and it magically works.

### End Goal

Framework to build fuzzers on the fly specialized for Tauri projects

- Specialized / White Box / Mutation based?
- LibAFL choice of tools
  - more customization for Tauri
  - long-term taint tracking analysis

### Sub-goals

- Automatized search for Tauri app entrypoints to the backend
  - Tauri API
  - custom commands

- Coverage guided fuzzing for Tauri
  - instrument Tauri application binaries to be suitable for coverage fuzzing
  - either during compilation
  - either during link-time optimization

- Fuzz the sample Tauri app using Qemu and snapshots

- Intercept IPC between Tauri and the webview 
  - directly call Tauri commands without using the webview

- Define interesting metrics for security
  - crashing, time, ...

- Improve performance of the fuzzing by having a mock runtime for Tauri
  - Mock runtime will enable us to fuzz a Tauri app without spawning a webview

### Open Questions

- How can we find the correct function symbols (mangling/no_mangling)?
- How can we observe filesystem changes?
- How can we observe network requests?

### Why do we use QEMU? 

We choose to fuzz the Tauri App in a VM using QEMU for 
- Tauri can safely interact with the OS 
- Using QEMU allow us to use snapshots
    - We can fuzz from a snapshot where Tauri has already been initialized 
    - We can fuzz the Tauri app from states that were deemed interesting

#### Disadvantages 

Fuzzing using QEMU will surely degrade performance.

#### Alternatives

- bare-metal with a Tauri mock runtime

### TODO

- [ ] Sample mini app crash fuzz

## Resources and References

- [LibAFL](https://github.com/AFLplusplus/LibAFL)
- [Fuzzers Like Lego (CCC Talk)](https://aflplus.plus/rC3_talk_2020.pdf)
- [Tauri Commands Documentation](https://docs.rs/tauri/latest/tauri/command/index.html)
- [LibAFL paper from 2022](https://www.s3.eurecom.fr/docs/ccs22_fioraldi.pdf)
- [Fuzzy 101](https://epi052.gitlab.io/notes-to-self/blog/2021-11-01-fuzzing-101-with-libafl/)
