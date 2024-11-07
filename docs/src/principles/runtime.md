# Runtime

In this section we explain how our runtime monitors interactions between the fuzzed application and
the host system.

## Concept

The concept of our runtime is simple; **the runtime monitors calls to a specific set of functions
of the target program during fuzzing**.
The set of functions monitored is chosen based on the policy provided by the user.
`tauri-fuzz` runtime has fine-grained monitoring. It can not only detect function calls of target functions
but can also inspect their parameters during a call or the return value when returning from them.

### Example

![Runtime monitors calls to `open` and `open64`](../images/runtime_monitors_access_to_foo.drawio.svg "Runtime monitors calls to `open` and `open64`")

For example, a user provides a policy which forbids interactions with the file named `foo.txt`.
On Linux, the runtime will start monitoring the `libc` functions that are mandatory
to interact with the file system access which are `open` and `open64`.
Moreover since the policy provided specifies that we want to block access to `foo.txt` the runtime
will only block calls to `open` and `open64` where `foo.txt` is the target file.

## Frida

We use the binary instrumentation toolkit [Frida](https://frida.re/) to monitor function calls.
[Frida interceptors](https://frida.re/docs/javascript-api/#interceptor) are used to inspect: arguments of function calls or return value of function return.
The reasons why we used Frida are two folds:

- Frida works on multiple platform: Linux, Windows, MacOS, Android, iOS. So `tauri-fuzz` can also be cross-platform.
- [LibAFL](https://github.com/AFLplusplus/LibAFL) a state-of-the-art fuzzer also has integration with Frida. This allows us to build a performant fuzzer through LibAFL
  which shares the same binary instrumentation toolkit with our runtime.
