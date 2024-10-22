# tl;dr of `tauri-fuzz`

**Observation**

Fuzzing is not used during application development

**Why?**

- Fuzzing requires time and experience to obtain results
- Fuzzers mostly detect memory corruption and crashes which are less relevant for applications

**How do we try to solve this?**

- Make fuzzing as easy as possible with `tauri-fuzz-cli` that can start fuzzing a Tauri application with few commands
- Provide a runtime that monitors the interactions between an application and to its host system.
  The runtime will block unsafe interactions which are defined by a provided policy.
- Provide a generic policy `no_error_policy` that can be used for all applications.
  `no_error_policy` will block and report any interactions with the host system that result into an error.
  Hence this policy detects application vulnerabilities that can be exploited to gain illegal access to the host system.
