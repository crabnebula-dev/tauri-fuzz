# tl;dr of `tauri-fuzz`

**Observation**

Fuzzing is not used during application development

**Why?**

- Fuzzing requires time and experience to obtain results
- Most fuzzers only detect memory corruption and crashes which are less relevant for applications

**How do we try to solve this?**

- Make fuzzing as easy as possible with `tauri-fuzz-cli` that can start fuzzing a Tauri application with few commands
- Provide a runtime that monitors the interactions between an application and its host system.
  The runtime will block unsafe interactions which are defined by a provided policy.
- Provide a generic policy `no_error_policy` that can be used for all applications.
  `no_error_policy` will block and report any interactions with the host system that result into an error.
  Motivation behind the `no_error_policy` is that if an application enables errors to happen when interacting with system resources
  then a malicious attacker could potentially exploit the application to control the system resources to its advantage.
