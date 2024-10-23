# The Fuzzer

In this section we explain how our runtime and policies are integrated into LibAFL.

## [LibAFL](https://github.com/AFLplusplus/LibAFL)

For simplicity `tauri-fuzz` provides a
[default implementation of a fuzzer](https://github.com/crabnebula-dev/tauri-fuzz/blob/main/crates/tauri-fuzz/src/fuzzer.rs)
which is built using [LibAFL](https://github.com/AFLplusplus/LibAFL).
LibAFL is a framework to build a fuzzers and integrate state-of-the-art tools to do so.

Moreover LibAFL has a crate [`libafl_frida`](https://github.com/AFLplusplus/LibAFL/tree/main/libafl_frida) to build Frida-based fuzzers.
These fuzzers possess features to improve fuzzing efficiency such as code coverage or logging of conditional statements.
Since our runtime is also based on Frida, integration of our runtime with `libafl_frida` is simpler and our default fuzzer benefits
from the performance of LibAFL.
This also gives us the possibility to fuzz our applications in the platforms supported by Frida: Linux, Windows, MacOS, Android and IOS.

## Can we use other fuzzers?

While LibAFL and `tauri-fuzz` are both using Frida they still use different parts of it.
`tauri-fuzz` uses [Frida Interceptors](https://frida.re/docs/javascript-api/#interceptor) to monitor function calls while `libafl_frida` uses [Frida stalker](https://frida.re/docs/javascript-api/#stalker) to do dynamic code instrumentation.
Therefore we believe it's possible to provide a variant of our runtime that could work with other fuzzers without too much issues.

This has not been investigated and is still work in progress so take these claims with a pinch of salt.
