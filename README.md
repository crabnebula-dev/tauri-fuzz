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

### Sub-goals for the day

- Automatized search for Tauri app entrypoints to the backend
  - Tauri API
  - custom commands

- Coverage guided fuzzing for Tauri
  - Rust code for the Tauri side
  - Include project crates

- Define interesting metrics for security
  - crashing, time, ...

### Open Questions

- How can we find the correct function symbols (mangling/no_mangling)?
- How can we observe filesystem changes?
- How can we observe network requests?

### TODO

- [ ] Sample mini app crash fuzz

## Resources and References

- [LibAFL](https://github.com/AFLplusplus/LibAFL)
- [Fuzzers Like Lego (CCC Talk)](https://aflplus.plus/rC3_talk_2020.pdf)
- [Tauri Commands Documentation](https://docs.rs/tauri/latest/tauri/command/index.html)
- [LibAFL paper from 2022](https://www.s3.eurecom.fr/docs/ccs22_fioraldi.pdf)
