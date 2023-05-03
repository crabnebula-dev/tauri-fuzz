# fuzzy-hackaton

Session to start integrating fuzzy testing for Tauri projects

## Schedule

1. Quick presentation about state of the art fuzzing
2. Fuzzing specialized for Tauri
3. Goals of the session

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

### End Goal

Framework to build fuzzers on the fly specialized for Tauri projects

- Specialized / White Box / Mutation based?
- LibAFL choice of tools
  - more customization for Tauri
  - long-term taint tracking analysis

### Sub-goals for the day

- Automatized search for Tauri app entrypoints to the backend
  - Tauri API
  - custom commands

- Coverage guided fuzzing for Tauri
  - Rust code for the Tauri side
  - Include project crates

- Define interesting metrics for security
  - crashing, time, ...

## Resources and References

- [LibAFL](https://github.com/AFLplusplus/LibAFL)
- [Fuzzers Like Lego (CCC Talk)](https://aflplus.plus/rC3_talk_2020.pdf)
- [Tauri Commands Documentation](https://docs.rs/tauri/latest/tauri/command/index.html)
