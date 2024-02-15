# Tauri-Fuzzer

## The fuzzer

### What is a fuzzer

A fuzzer is an automatic testing tool commonly used for softwares.
The goal is to test your software by executing it with a large set of pseudo-randomly generated inputs.

### What's special about this fuzzer

- Specializes in testing a system security boundaries rather that looking for memory errors
- Code is fuzzed against a security policy
    - Some default ones are provided
    - Users can provide custom policies
- Built on top of [LibAFL](https://github.com/AFLplusplus/LibAFL) and [Frida](https://frida.re/)
    - Portable on Windows, MacOS, Android, iOS

Additional information can be found in the mdbook in `/docs`.

## Repository Architecture

- `mini-app` is a minimal Tauri application used to test and demonstrate the fuzzer
- `mini-app-fuzz` is where the fuzz environment for `mini-app` is setup
- `tauri-fuzz-tools` is a crate providing utilities
- `docs` contains technical information and thoughts process behind the project

## Biblio on fuzzing

Technical documentation, research and thoughts process that happened during the development of this project are documented in the mdbook in `docs`.

Requires `mdbook` and `mdbook-toc`

```bash
$ cargo install mdbook
$ cargo install mdbook-toc
```

## Installation

### Requirements

Tauri dependencies:
- libwebkit2gtk-4.0-dev
- build-essential
- curl
- wget
- file
- libssl-dev
- libgtk-3-dev
- libayatana-appindicator3-dev
- librsvg2-dev

Fuzzer dependencies:
- libc++-15-dev
- libc++abi-15-dev
- clang-15


### Setup a VM for fuzzing

Fuzzing may be harmful for your system.
Especially in this case where the fuzzer try to shell execution, file system corruption, ...

We provide a Debian VM in [virtual-machines repo](https://github.com/crabnebula-dev/virtual-machines) in the `feat/tauri-fuzz` branch.

#### Generate and connect to the VM:
- Go to the `virtual-machines/01_tauri_fuzz/debian` directory
- Build and start the VM with `make run`
- You can connect to the VM using SSH using port 2222
    - username is `user`
    - password is `user`
    - `ssh -p 2222 user@localhost`

#### Setup Tauri fuzzing tools in the VM

After connecting to the VM using SSH
- __Requirements__:
    - be part of CN github organization
    - having registered an SSH key in Github
    - having SSH key port forwarding for Github private CN repo
- Execute the script at `/home/user/setup_fuzz_tools.sh`
- This will download and compile:
    - `tauri-fuzzer` this repo
    - CN private fork of `LibAFL`

## Test the fuzzer on `mini-app`

We recommend using a VM for testing.
Instructions to get one are above.

Go to the `mini-app/src-tauri/fuzz` directory.

### Running the test

At the root of `tauri-fuzzer`.

```
cargo test
```

### Running specific fuzz targets

Fuzz targets are in the `mini-app/src-tauri/fuzz/fuzz_targets` directory.
Any of the fuzz targets can be executed in this directory with
```
cargo r --bin <fuzz target name>
```

## Resources about the fuzzer

LibAFL:
- https://aflplus.plus/libafl-book/baby_fuzzer.html (book)
- https://www.youtube.com/watch?v=L7BaCIciFEM (orginal video)
- https://media.ccc.de/v/37c3-12102-fuzz_everything_everywhere_all_at_once (more recent video, with topics close to us)
- https://www.s3.eurecom.fr/docs/ccs22_fioraldi.pdf (research paper)
