# Tauri-Fuzzer

Tauri-Fuzzer is used to fuzz Tauri applications

## Architecture

- `mini-app` a minimal Tauri application which is the fuzz target
- `fuzzer` directory containing the custom fuzzer
- `docs` contains information about bibliography and advancement of project

## Biblio on fuzzing

In the mdbook in `docs`.

Requires `mdbook` and `mdbook-toc`

> cargo install mdbook
> cargo install mdbook-toc

## Testing the fuzzer

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
    - `cargo-tauri-fuzz` tools to fuzz programs
    - `tauri-fuzzer` this repo

### Test the fuzzer on `mini-app`

We recommend using a VM for testing. 
Instructions to get one are above.

Go to the `mini-app/src-tauri/fuzz` directory.

#### Fuzzing targets

Fuzz targets are in the `fuzz_targets directory`.
They can also be listed with 

```
cargo-tauri-fuzz list
```

#### Fuzz a target 

```
cargo-tauri-fuzz run {target name}
```

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

