# Application Fuzzing

The project aims to provide fuzzing as an automatic testing tool for applications.
In this section we describe what we think is necessary to popularize the use of fuzzing in application development.

## Current state of fuzzing

From our knowledge a majority of fuzzers use memory sanitizers and crashes from the tested software to detect issues during the fuzzing process.
This is really effective when fuzzing C/C++ code since it is evaluated that
[70% of found vulnerabilities in those languages are memory safety bugs](https://storage.googleapis.com/gweb-research2023-media/pubtools/pdf/70477b1d77462cfffc909ca7d7d46d8f749d5642.pdf).

However application development are usually done with technologies which are less impacted by memory errors.
Therefore usual fuzzing techniques are not suited to test applications.

Another reason where fuzzing is not often used in the applicative world is that fuzzing requires
domain knowledge (regarding fuzzing and the tested program) to setup and obtain results.
While it makes sense to spend time to fuzz libraries that are shared between numerous projects it is not
clear that fuzzing applications is cost effective.

## Enabling fuzzing to applications

We believe that popularizing an automatic testing tool such as fuzzing during application development could improve
the current state of application security overall.
As we said above fuzzing is not integrated into application development for two main reasons: complex to use and not suited.

### Make fuzzing Tauri applications as easy as possible

We build a CLI tool `tauri-fuzz-cli` that tries to make fuzzing as easy as possible for Tauri applications.
This CLI does two things:

- setting up an environment for fuzzing in a Tauri project with one command
- fuzzing a [Tauri command](https://v2.tauri.app/develop/calling-rust/#commands) with one command

`tauri-fuzz-cli` is inspired from [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz).

More details on `tauri-fuzz-cli` can be found in the next chapter.

### Fuzzing meaningful safety properties

We propose a different fuzzing technique which **tests the security boundaries of an application**.
Applications are executed on a host system (laptops, smartphones...) which exposes different shared resources to these applications.
An example of such resources are host file system, shell, network...
The idea behind it is that we want to avoid situations where malicious attackers are able to leverage an application vulnerabilities to access
more of the system resources than what the application is supposed to do.
To summarize the goal of our fuzzing project is to **detect vulnerabilities that allow an attacker to access more of the system resources
than intended**.

### Example: detect illegal interactions with the file system

![Fuzzing applications security boundaries ](../images/fuzzing_application_boundary.drawio.svg "Fuzzing applications security boundaries")

Here is an example. We want to test an application that is able to interact with the network but is not supposed to interact with the file system.
We give to `tauri-fuzz` a policy that the application _should not interact with the file system_.
Hence while being fuzzed, any interactions with the file system will be blocked and reported to the tester. However since we
did not tell our fuzzer to monitor interactions with the network, these will be accepted by the fuzzer.

The next section will provide information on how these interactions with the host system are monitored and how can a developer provide a policy.
