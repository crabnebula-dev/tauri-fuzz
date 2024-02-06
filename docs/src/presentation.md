# Presentation

## Intro: What is fuzzing and how is it used

### How to test a software

1. Manual testing
    - Concrete input and test if you get the expected output
    - Easy to setup but does not cover a lot of cases
    - Used in every serious software project
2. Fuzzing
    - Multitude of input and test if the specified invariant is respected
    - Requires a bit of setup and not easily applicable to all software
    - Mostly used in software that are
3. Formal verification
    - Prove that for any input the program respect a certain property
    - Time consuming and requires human expertise
    - Only used for critical software

### Challenges of the fuzzer

- How do you cover the maximum number of states?
- How do you know that an execution went wrong?
    - Example: monkey testing an application UI
    - The most used invariant is crash detection and memory corruption
    - Many critical software are written in C and are prone to memory errors
    - crashes are fatal in: Space, aeronautics,...

### Fuzzer for applications

- Fuzzers are not often used in the applicative world
- Most applications are not that interested by crash
    - they don't use C so memory errors do not really happen
    - crashing is not the biggest security issue and you are more concerned about information flow
- There is
    - UI fuzzing, don't know much about it but how do you automatically  detect if the UI is is a wrong state
    - URLs, forms fuzzing but it's also hard to automatically detect if something went wrong
- Problematic: App fuzzing requires insider knowledge and contribution to be interesting

### Tauri-fuzzer is a fuzzer specialized for application testing

- An application can be modeled as a function that computes an output based on resources containing the data
- We want to make sure that an app does not access more resources that necessary
- A security policy describes what resources should be allowed to access
- Our fuzzer is given a security policy and test if the policy is respected

### Goal of the `tauri-fuzzer`

- Facilitate the usage of fuzzing in the applicative world
- Must be as easy as possible to integrate in a developer workflow
- Easily configurable to fine-tune the fuzzer to the targeted application

## Architecture of the fuzzer for a Tauri application

### General architecture

Schema:
- Tauri frontend: assumed malicious
- Tauri backend is the main access to the system resources
- Fuzzer:
    - is emulating the frontend
    - controlling access to system resources
- Security policy is given to the fuzzer

### More technical details on the fuzzer

- Fuzzer is using LibAFL:
    - parallelized fuzzing
    - code coverage
    - statistics
- Access to system resources is controlled with Frida:
    - Modify the binary of the targeted program
    - We attach a listener to every function we want to monitor
        - check the arguments
        - check the return value
- Security policy is defined and sent to the fuzzer:
    - which function we want to monitor
    - which arguments are accepted
    - which return value

## Fuzzing your Tauri application made as easy as possible

### Default security policies

Writing a security policy can be annoying and requires time.
We provide two default security policies:
- Any dangerous utilities should not return an error status
    - shell, sql, networking ...
    - fuzzer is likely to discover unsafe usage of these utilities through a syntax error
    - this is good against insufficient input validation
- Derive a security policy from the Tauri configuration file
    - the allowlist defines what the Tauri frontend is allowed to access
    - we think this can be used as an approximation of the Tauri app security policy

### Generate fuzz target automatically

A fuzz target is glue code that is given to the fuzzer and that will be executed repeatedly.
This is usually written by the app developer since it requires knowledge about the app API.
- Analyze the Tauri project and produce a fuzz target per Tauri command
    - parse the Tauri code and find Tauri macros
    - retrieve function signature to invoke it correctly in the fuzz target
- Using AI to generate fuzz targets
    - Currently the most studied technique in fuzzing research

## Conclusion and summary

__Goal__ popularize the usage of fuzzers for applications
- Make fuzzing useful for applications by testing their security policy
- Make fuzzing Tauri application as easy as possible
    - two default security policies
    - automatic generation of fuzz targets

__Biggest issue__
This work is a prototype and has not been tested thoroughly

__Future work__
- Code generation
- Port to other platforms
- Testing

## Demo
