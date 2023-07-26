# Fuzzy prototype

Fuzzer prototype to use for Tauri applications

## Architecture

- `mini-app` a minimal Tauri application which is the fuzz target
- `fuzzer` directory containing the custom fuzzer
- `docs` contains information about bibliography and advancement of project

## State of the Art Fuzzing

- Architecture of a Fuzzer
- Types of fuzzers
  - Black/Grey/White box
  - Mutation/Generation based
  - Generalized/Specialized
- Popular fuzzers: AFL, libFuzzer, hongfuzz
- LibAFL framework
- Areas of research/improvement
  - roadblock bypassing
  - structure aware fuzzing
  - corpus scheduling
  - energy assignment

## Test the fuzzer

To fuzz the `mini-app` in the repo on the command `tauri_cmd_2`.
This command is supposed to crash when given the input `100u32`.

### Run the fuzz

#### Locally

In the `fuzzer` directory type:
> `cargo run`

#### With Docker

The `Dockerfile` is meant to run the fuzzer. The idea is to use Vscode DevContainers for
the fuzzer, as libAFL has some issues on some distros.

`docker build . -t test-fuzz`

`docker run -it --privileged test-fuzz`

`cd fuzzer`

`cargo build --release`

OR

Use the devcontainer feature from vscode and it magically works.

### Check the fuzzing results

Outputs from the fuzzer are stored in files in the `fuzzer/crash/` directory.
Each file represents an input on which the tested command has crashed.

View the input value using:
> `hexdump -C crashes/file_in_the_dir`

The result should contain this:
> 00000000  00 00 00 64 35 ff ff ff

The 4 first bytes represent the `u32` that was given to the tested command.
In hex `0x64 = 100` which is the input on which the tested command crash.


## Tauri Fuzzing

### End Goal

- Framework to build fuzzers for Tauri apps
- Fuzzer for Tauri itself
    - custom protocol
    - backend/frontend communication
    - configuration

- Specialized / Grey Box / Mutation based
- LibAFL choice of tools
  - more customization for Tauri
  - long-term taint tracking analysis

## Step to fuzz the commands of a Tauri app

1. Turn the Tauri app into a lib
  - Add a `src-tauri/src/lib.rs` file in the Tauri app
  - Turn Tauri commands visibility to `pub`
  - Allow public re-export of Tauri commands by adding in the `lib.rs` file
    - `pub mod file_where_commands_are`
    - `pub use file_where_commands_are::*`
2. Import the Tauri application as a crate in your Cargo file
3. Code `InvokePayload` creation specific to each Tauri command
  - examples are `crate::tauri_fuzz_tools::payload_for_tauri_cmd_2` and
    `fuzzer::tauri_fuzz_tools::payload_for_tauri_cmd_1`
4. Change the harness in `crate::fuzzer::in_process()` function
  - use your payload creation function you just wrote

## Documentation

In an `mdbook` format.

Requires `mdbook` and `mdbook-toc`

> cargo install mdbook
> cargo install mdbook-toc


## Tips

### Avoiding wear and tear of physical disk

When using afl, you can transfer the heavy writing to RAM
>  docker run -ti --mount type=tmpfs,destination=/ramdisk -e AFL_TMPDIR=/ramdisk aflplusplus/aflplusplus

### Improving fuzzing speed

Section 3.i of
[AFL Guide to Fuzzing in Depth](https://github.com/AFLplusplus/AFLplusplus/blob/stable/docs/fuzzing_in_depth.md)

- Use persistent mode (x2-x20 speed increase).
- If you do not use shmem persistent mode, use AFL_TMPDIR to point the input file on a tempfs location, see /docs/env_variables/.
- Linux: Improve kernel performance: modify /etc/default/grub, set GRUB_CMDLINE_LINUX_DEFAULT="ibpb=off ibrs=off kpti=off l1tf=off mds=off mitigations=off no_stf_barrier noibpb noibrs nopcid nopti nospec_store_bypass_disable nospectre_v1 nospectre_v2 pcid=off pti=off spec_store_bypass_disable=off spectre_v2=off stf_barrier=off"; then update-grub and reboot (warning: makes the system more insecure) - you can also just run sudo afl-persistent-config.
- Linux: Running on an ext2 filesystem with noatime mount option will be a bit faster than on any other journaling filesystem.
- Use your cores! See 3c) Using multiple cores.
- Run sudo afl-system-config before starting the first afl-fuzz instance after a reboot.

