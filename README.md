# Tauri-fuzz

This is a runtime environment to use when fuzzing.
This runtime is specialized in detecting security boundaries violations in applications.

## The project

### What is a fuzzer

A fuzzer is an automatic testing tool commonly used for software.
The goal is to test your software by executing it with a large set of pseudo-randomly generated inputs.

### What's special about this fuzz runtime

Most fuzzers are dedicated to find memory bugs in C libraries.
In our case we focus on security issues in applications.
Specifically we check that applications can't break their assumed security boundaries.

#### Examples of cases where the runtime is relevant

In general the runtime is useful to check the security boundaries of an app:

- an app should have no or limited access to the filesystem
- an app has access to the shell but we want to make sure that it cannot be abused
- an app should not make any remote connection except to specified servers (TODO)

#### Summary

- Specializes in testing applications security boundaries
- Target code is fuzzed against a security policy
  - Several default policies are provided
  - Users can provide custom policies
- Cross-platform fuzzing
  - Built on top of and [Frida](https://frida.re/)
  - Coupled to [LibAFL](https://github.com/AFLplusplus/LibAFL) for state of the art fuzzing
  - Portable on Windows, MacOS, Android, iOS (TODO)

### Main default policy

Several default policies are provided but one policy in particular stands out and can be useful almost anywhere:

> The target code is not allowed to call external binaries in a way that the external binaries will return an error

The assumption is that if an app is able to call a binary with inputs such that the binary returns
an error then an attacker has the room to exploit this binary call to mount an attack.
When fuzzing if such vulnerability appears it will likely be under the form of a syntax error due to the random nature of fuzzing.

## Repository Architecture

- `crates/tauri-fuzz-cli` a cli to initialize fuzzing in a project
- `crates/tauri-fuzz` the runtime used while fuzzing
- `crates/tauri-fuzz-policies` the security policies and the policy engine that will be used while fuzzing
- `docs/` technical information and thoughts process behind the project
- `examples/` examples to run the fuzzer on
- `tests/` tests

## Documentation

Technical documentation, research and thoughts process that happened during the development of this project are documented in the mdbook in `docs`.

Requires `mdbook` and `mdbook-toc`

```bash
$ cargo install mdbook
$ cargo install mdbook-toc
```

## Installation

### Requirements

Requirements for the fuzzer dependencies on [LibAFL repo](https://github.com/AFLplusplus/LibAFL)
Requirements for Tauri fuzzing on [Tauri website](https://tauri.app/start/prerequisites/)
