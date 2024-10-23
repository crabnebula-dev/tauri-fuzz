# [`tauri-fuzz`](https://github.com/crabnebula-dev/tauri-fuzz)

The goal of this project is to provide a tool to easily fuzz [Tauri](https://tauri.app/) applications.
[`tauri-fuzz`](https://github.com/crabnebula-dev/tauri-fuzz) fuzzes your Tauri app with a special runtime that detects when security boundaries are breached.
By security boundaries we mean unsafe interactions with the host system resources.

![Fuzzing applications security boundaries ](./images/fuzzing_application_boundary.drawio.svg "Fuzzing applications security boundaries")

> **[Disclaimer]** `tauri-fuzz` was tailored to be used with Tauri applications but the fuzzing principles should be
> reusable to fuzz other types of applications.

## Origin of the project

Applications are now a growing part of our daily lives.
Many vulnerabilities are present in them which can be used to harm the users.
To minimize such vulnerabilities developers need to thoroughly test their applications.
One of the most popular way to automatically test your software is called fuzzing.

The principle of a fuzzer is to test a software by executing it with a very large amount of semi-random inputs and to detect any problematic behaviours during these runs.
Currently most fuzzers are used to detect memory safety vulnerabilities for popular C libraries.

### Why are fuzzers not used for applications?

We see two main reasons:

- Fuzzing can be hard to setup and requires experience and/or time to be used effectively.
- Applications are often developed with technologies that are less prone to memory vulnerabilities. So
  fuzzer default error detection mechanisms do not translate well for applications.

### Goal of the project

This project aims to fill this gap and provide fuzzing to applications:

- We try to facilitate as much as possible the process of fuzzing for Tauri apps
- We provide a cross-platform runtime that detects any behaviour that breaches the security boundaries of a fuzzed application

## Components of `tauri-fuzz`

More details can be found in the [Principles](./principles.md) section.

### `tauri-fuzz`

`tauri-fuzz` contains the runtime which is used during fuzzing to detect whenever the application interacts with external components
It uses [`Frida` interceptors](https://frida.re/docs/javascript-api/#interceptor) to monitor any interactions to a set of system resources specified by a provided fuzz policy.
It also contains utilities that facilitate the integration with the web application framework [`Tauri`](https://tauri.app/) and the fuzzer framework [`LibAFL`](https://github.com/AFLplusplus/LibAFL).

### `tauri-fuzz-policies`

A framework to use and create security policies used by the runtime during fuzz time.
A fuzz policy defines the security boundaries that will be enforced by the runtime on the application while being fuzzed.
For example we can provide to `tauri-fuzz` a policy that prevents any interaction with the file system.
In this configuration, anytime the fuzzed application try to use the file system it will get intercepted by the runtime and get reported as a security breach.

### `tauri-fuzz-cli`

A command line utility that simplifies as much as possible the steps to fuzz a Tauri app.
It handles both setting up the fuzzing environment and starting fuzzing instances for a Tauri app.
