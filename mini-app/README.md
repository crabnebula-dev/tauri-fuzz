# Mini-app

This is a minimal Tauri app that is used for testing Tauri fuzzing.

## `mini-app` vulnerabilities

We want the fuzzer to be able to test for:
- shell command injection
- file access and corruption
- sql injection

Commands containing these kind of vulnerabilities are implemented as Tauri `command` in `mini-app`





