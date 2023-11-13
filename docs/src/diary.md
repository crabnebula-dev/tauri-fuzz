## 1
Playing with the Tauri mock runtime
- Exploration of Tauri code
    - `tauri::app::App::run_iteration` exists to react to a single event
    - `tauri::app::on_event_loop_event` could be used to fuzz the Tauri app by calling it directly from main
    - `tauri::app::Builder::on_page_load` could be used to fuzz the Tauri app by calling it directly from main
    - `tauri::tauri-runtime-wry` is the default implementation of the runtime
    - `tauri::tauri-runtime` is the runtime interface
    - `wry-runtime` event loop receives different type of events:
        - `tao::Event` receives from TAO
        - `tao::EventLoopWindowTarget` ?
        - `tao::ControlFlow`: Poll, Wait, WaitUntil, Exit

- playing with `mini-app` and mock runtime
    - new fuzz branch in Tauri
    - make the mockruntime public
    - rust-gdb can be used to break on points such as: `tauri::app::App::run_iteration::hc9a795e571e144bc`
    - trying to hack the function `tauri::app:on_event_loop_event`
        - events are only for window stuff, to interact with command check manager

## 2

- try to trigger a command programatically
- `PendingWindow` has fields
    - `js_event_listeners`
    - `ipc_handler`
- Check `wry` crate
    - webview sends message to the Rust host using `window.ipc.postMessage("message")`
- Try to capture IPC using wireshark
    - listening on the loopback interface
    - did not work, certainly __tauri does not use internet socket__
- Try to capture IPC using `strace`
    - we see traces of `recvmsg` and `sendmsg` syscalls
    - using `ss -pO | grep mini/WebKit` we see existences of open sockets for these processes
    - Unix sockets can be tracked using this [sockdump](https://github.com/mechpen/sockdump)
        - `sockdump` can output to pcap format that is readable by wireshark

## 3

- Trying to `sockdump` the mini-app sockets
    - checking sockets file in `/proc/$PID/fd`
    - `lsof -p $PID` lists open files for a process
    - __tauri command does not seem to pass through unix sockets__
        - `ss` show that the open sockets have no data going through them
        - this is confirmed using `sockdump`
- Checking `tauri`, `wry` and `tao` code to see where IPC comes from
    - connect to local version of wry and tauri
    - `tao::EventLoop::run_return` when spawning x11 thread contains
      ` let (device_tx, device_rx) = glib::MainContext::channel(glib::Priority::default()); `


## 4

- IPC manager add to Webkit IPC handlers
    - at build time of the webview these handlers will generate methods
      that can called via `window.webkit.messageHandlers.funcName.postMessage(args)`
    - examples can be seen in `wry/examples/*`
- From Lucas suggestion
    - `tauri::window::Window::on_message` can trigger command
    - `https://github.com/tauri-apps/tauri-invoke-http` to use http over localhost instead of default Tauri
- Using `tauri::window::Window::on_message` we manage to run the app and trigger command without webview

## 5

- import tauri-fork in the fuzz-proto dir
- reinstall necessary tools for new computers
- modify Dockerfile
    - remove `cargo chef`, don't know why but it made `mini-app/src-tauri/src/main.rs` an empty `main(){}` function
    - change the architecture

## 6
- modify Dockerfile to have missing dependencies
- `tauri::test::assert_ipc_response` should be checked to also handle response from the command invoked

### Question to Lucas
- IPC lifecycle?
    - on init of webview, tauri register in the webview tauri handles
    - this tauri handles can be called via `postMessage` in webkitgtk
    - What kind of Linux IPC are actually used in webkitgtk
  > ipc are actually handled by the webview
- Mockruntime
    - essentially what is it? emulation of Wry
    - if we want to fuzz the windowing system in the future could it be interesting
  > fork the mockruntime if you want to fuzz the windowing system rather than forking wry
- HTTP
    - does it make the 2 process communicate over http on localhost
    - when is it used?
    > websockets, local devserver
  > could be useful for a man-in-the-middle fuzzer that is able to fuzz both the backend and
  > the webview by sending them HTTP requests
- Architecture questions
    - why do use Window and WindowHandle, App and AppHandle

## 7
- `libdw` is not used in `cargo b --release` because there are no debug info in release profile
- fix byte conversion error were the `copy_from_slice` involved 2 arrays of different sizes
- `libafl::bolts::launcher::Launcher` is used to launch fuzzing on multiple cores for free
    - `run_client()` is the closure ran by every core
- Fuzzer behaviour depending on harness result
    - When harness crashes with `panic`
        - the fuzzer state is restarted
        - re-generating initial corpus
    - When harness does not crash but return `ExitKind::Crash` or `ExitKind::Ok`
        - the fuzzer is not restarted and corpus may ran out because not regenerated
- `libafl::state::StdState::generate_initial_inputs_forced` create new inputs even if they are not "interesting"
    - useful when not using feedback

## 8
- x86\_64 calling convention checked
    - for `&str` length is store in rsi and pointer in rdi
    - for `u32` value is stored directly in rdi
- environment variable `LIBAFL_DEBUG_OUTPUT` helps with debugging

## 9
- `libdw` issue
    - In the docker container it works in release but not in debug
    - In local it does not work in both release and debug and this issue is triggered in both cases
- `libafl_qemu::Emulator` does not crash itself when the emulated program crash
    - no way to catch a crash in the emulator?
- Add `InProcess` fuzzing
    - we avoid the dependency issue
    - we don't deal with qemu emulator anymore
    - steps
        1. Split `mini-app` to have both a binary and a lib
        2. Use the in-memory fuzzing to call functions from the lib
- separate mini-app into a lib and binary

## 10
- Flow between app and mockruntime
    - `app::run()` - `runtime::run()` - `app::on_event_loop_event` - `callback`
- diff between:
    - `App::run_on_main_thread`/`RuntimeContext::run_on_main_thread`, run stuff on the window process
    - `window::on_message`: pass message to backend process
- need to have a harness that does not exit at the end of the function
- In the `mockruntime` there is `app::Window::DetachedWindow::Dispatcher::close()`
    - it will send the message `Message::CloseWindow` with `run_on_main_thread`
    - the mockruntime intercept it and sends `RuntimeEvent::ExitRequested` to the `app`
    - the `app` will process some stuff in `on_event_loop_event`
    - then the event `RuntimeEvent::ExitRequested` will be sent to the closure given to `app::run` at the beginning
- you can break out of the loop from `run` in the `Mockruntime`
    - by sending a message `Message::CloseWindow`
    - then sending another message which is __not__ `ExitRequestedEventAction::Prevent`

## 11 
- Move code that setup and calls tauri commands to the fuzzer 
    - now the application can add an almost empty `lib.rs` file to 
      to be fuzzed
- Refactor and clean code
- Bibliography
    - tinyinst

## 12
- Bibliography 
- Mdbook
- Plan for the future with Github issues

## 13 
- Read AFL++ docs for code instrumentation 
- Redo the dockerfile 
    - Change to higher version of Debian to have llvm14 - Fail, llvm14 is not new enough to compile rust code
    - Change to Ubuntu container 23.04
    - Pin the Rust version to 17.0
    - Pin compiler version for AFL++ to llvm-16
- Compile with `afl-clang-lto`
    - version of rustc llvm and the llvm you want to use need to match
        - check your rustc llvm with `rustc --version --verbose`
    - output llvm with `rustc` + vanilla compilation with `afl-clang-lto`
    fails and not practical
    - trying with `.cargo/config.toml`
        - `[target.x86_64-unknown-linux-gnu] linker = "afl-clang-lto"`
- Checking if coverage worked by checking asm
- `afl-clang-lto` needs more instrumention before in the pipeline
- we need to check `cargo-afl`

## 14 
- in `cargo-afl` 
    - files are compiled with
    `let mut rustflags = format!(
        "-C debug-assertions \
         -C overflow_checks \
         -C passes={passes} \
         -C codegen-units=1 \
         -C llvm-args=-sanitizer-coverage-level=3 \
         -C llvm-args=-sanitizer-coverage-trace-pc-guard \
         -C llvm-args=-sanitizer-coverage-prune-blocks=0 \
         -C llvm-args=-sanitizer-coverage-trace-compares \
         -C opt-level=3 \
         -C target-cpu=native "
    );
    rustflags.push_str("-Clink-arg=-fuse-ld=gold ");
    `
- Compile mini-app with the function above 
    - issue all crates are instrumented 
    - `export RUSTFLAGS="-C debug-assertions -C overflow_checks -C passes=sancov-module -C codegen-units=1 -C llvm-args=-sanitizer-coverage-level=3 -C llvm-args=-sanitizer-coverage-trace-pc-guard -C llvm-args=-sanitizer-coverage-prune-blocks=0 -C llvm-args=-sanitizer-coverage-trace-compares -C opt-level=3 -C target-cpu=native --cfg fuzzing -Clink-arg=-fuse-ld=gold -l afl-llvm-rt -L /home/adang/.local/share/afl.rs/rustc-1.70.0-90c5418/afl.rs-0.13.3/afl-llvm-rt"`
    - we need to make `-fsanitize-coverage-allowlist=` work

## 15
- Check `LibAFL`
    - `libafl_targets`
    - `libafl_cc`
- Compile with `-C llvm-args=-sanitizer-coverage-trace-pc-guard`
    - it place calls to `__sanitizer_cov_trace_pc_guard` at every edge (by default)
    - `libafl_targets` implements `__sanitizer_cov_trace_pc_guard`
    - flags
        - `export RUSTFLAGS="-C debug-assertions -C overflow_checks -C passes=sancov-module -C codegen-units=1 -C llvm-args=-sanitizer-coverage-level=3 -C llvm-args=-sanitizer-coverage-trace-pc-guard -C llvm-args=-sanitizer-coverage-prune-blocks=0 -C llvm-args=-sanitizer-coverage-trace-compares -C opt-level=3 -C target-cpu=native --cfg fuzzing -C llvm-artg=-D__sanitizer_cov_trace_pc_guard_init"`
    - `sanitize-coverage-allowlist=coverage_allowlist.txt` not supported with rust
    - linking error, `ld` does not find symbols in `libafl_targets`
- Selective instrumentation
    - try allowlist but not working
    - `cargo rustc`, which only affects your crate and not its dependencies.
        - https://stackoverflow.com/questions/64242625/how-do-i-compile-rust-code-without-linking-i-e-produce-object-files
- From Discord: 
    - "I had good experience with using cargo-fuzz and https://github.com/AFLplusplus/LibAFL/pull/981 together"
    - "So cargo-fuzz will instrument everything and that branch has a libfuzzer compatible runtime"
    - "In a default cargo-fuzz project, just depend on that LibAFL libfuzzer version instead of the one from crates.io."
    - "There is also the (somewhat unmaintained) cargo-libafl crate that could give some pointers"
- `rustc` llvm-args
    - `rustc -C llvm-args="--help-hidden" | nvim -`

## 16
- `cargo-libafl` is a fork of `cargo-fuzz`
- How does it work with libfuzzer 
    1. `init` command creates a `fuzz` directory with
        - `fuzz_targets` with harness using the `fuzz_target!` macro
        - `Cargo.toml` containing dependency to `libfuzzer-sys`
        - `libfuzzer-sys` can refer to the original from `crates.io`
        or to the ported version from `libafl`
    2. `cargo-fuzz run` command to fuzz the targets
        - Working when using the deprecrated original `libfuzzer-sys`
        - Failing to link with the version from `libafl` 
        - Same error when using `cargo-libafl`
        - Steps:
            1. Compile the `fuzz_targets` with the command
            `RUSTFLAGS="-Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-level=4 -Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-pc-table -Cllvm-args=-sanitizer-coverage-trace-compares --cfg fuzzing -Clink-dead-code -Cllvm-args=-sanitizer-coverage-stack-depth -Cdebug-assertions -C codegen-units=1" "cargo" "build" "--manifest-path" "/home/adang/boum/fuzzy/playground/rust-url/fuzz/Cargo.toml" "--target" "x86_64-unknown-linux-gnu" "--release" "--bin" "fuzz_target_1"`
            2. Run the `fuzz_targets` with the command
            `RUSTFLAGS="-Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-level=4 -Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-pc-table -Cllvm-args=-sanitizer-coverage-trace-compares --cfg fuzzing -Clink-dead-code -Cllvm-args=-sanitizer-coverage-stack-depth -Cdebug-assertions -C codegen-units=1" "cargo" "run" "--manifest-path" "/home/adang/boum/fuzzy/playground/rust-url/fuzz/Cargo.toml" "--target" "x86_64-unknown-linux-gnu" "--release" "--bin" "fuzz_target_1" "--" "-artifact_prefix=/home/adang/boum/fuzzy/playground/rust-url/fuzz/artifacts/fuzz_target_1/" "/home/adang/boum/fuzzy/playground/rust-url/fuzz/corpus/fuzz_target_1"`

- `fuzz_target!` macro definition is in `cargo-libafl/cargo-libafl-helper`  
- To have a more complete fuzzer with memory sanitizer and else check
`cargo-libafl/cargo-libafl/cargo-libafl-runtime`
- Fork `cargo-fuzz` or `cargo-libafl` to use their framework to easily fuzz Tauri applications

## 17
- Use `cargo-fuzz` as frontend for the fuzzing then use `libafl` as a backend replacing `libfuzzer`
- Installing `rustup component add llvm-preview-tools` to see information about code coverage
    1. `cargo fuzz run fuzz_target`
    2. `cargo fuzz coverage fuzz_target`
    3. Show code coverage with `llvm-cov show`
    > `llvm-cov show \
    -instr-profile=coverage/fuzz_target_1/coverage.profdata \
    -Xdemangler=rustfilt target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/fuzz_target_1 \
    -use-color --ignore-filename-regex='/.cargo/registry' \
    -output-dir=coverage/fuzz_target_1/report -format=html \
    -show-line-counts-or-regions \
    -ignore-filename-regex='/rustc/.+'`
        - docs on [https://llvm.org/docs/CommandGuide/llvm-cov.html#llvm-cov-show](https://llvm.org/docs/CommandGuide/llvm-cov.html#llvm-cov-show)
        - bin with coverage information is generated at `target/arch_triple/coverage/arch_triple/release/fuzz_target`
        - profile file is generated at `coverage/fuzz_target/coverage.profdata`
    4. Create a summary report with `llvm-cov report` 
    > `llvm-cov report \
    -instr-profile=coverage/fuzz_target_2/coverage.profdata \
    -use-color --ignore-filename-regex='/.cargo/registry' \
    -Xdemangler=rustfilt target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/fuzz_target_2`
- Swap `libfuzzer` backend with `libafl_libfuzzer` version
    - doc for options in the `LibAFL/libafl_libfuzzer/src/lib.rs`

## 18 
- Clone dash
- Clone sqlite
- Modify `dash` to make it crash 

## 19 
### Frida 
Frida is a binary analyser with 2 main features
    - _Stalker_ code-tracing engine
        - follow threads and trace every instruction 
        that are being called
        - uses a technique called _dynamic recompilation_
            - while a program is running the current basic block is copied and stored in caches
            - these copy can be modified and executed on demand
            - the original instructions are unmodified
    - _Interceptor_ hooks
        - allows inspection and modification of the flow of function calls
        - different possible techniques but most common are trampoline based hooks
        - code is inserted at the beginning of a function A to execute another function B so function B is "inserted" in the middle of function A

#### Strong points 

- Portability: frida works/exists on almost all platforms
- Frida is binary analysis 
    - works directly on binaries and do not require special compilation

#### Libafl-frida

- `libafl-frida` uses frida ability to modify the code to
    - provide coverage
    - provide asan 
    - provide cmplog
- to create more behaviour we just need to implement the `FridaRuntime` and add it to the possible runtimes
    - for example a runtime that crash on system call
- `libafl-frida` has been made to fuzz C libraries
    - no easy way to fuzz a Rust crate


## 20 

### Syscall isolation runtime

#### Intercepting syscalls
- using ldpreload trick 
- intercept all libc and `syscall` instruction

#### Preventing too many false positive 
- SET a flag every time you change of running environement (disable flag when running fuzzer code)
    - needs to be run single-threaded
- Check for stack trace to see if it came from the Tauri app
    - can be costly 
- Use fork fuzzing to not have syscalls from the fuzzer?
- EBPF could be a solution to filter false positive? There may be already existing ebpf rules that exist that we could reuse
- Using libafl minimizer

## 21 

### tauri-for-fuzzy 
- `window.close()` has different behaviour in 1.5 and 2.0

### Fuzzer on macos 
- `tauri::Window` other than "main" can't trigger `on_message`
- issue with using `Cores("0")` but works fine with other cores
    - `cores.set_affinity()` not supported for MacOS
    - I have a hunch that `Cores("0")` represent inmemory fuzzing

### Ideas for Frida
- For injecting library dependency on PE, Mach0 or ELF
- https://github.com/lief-project/LIEF

### Interesting project 
- [ziggy](https://github.com/srlabs/ziggy/tree/3b47b49ae7a822a0359ffabe3fa597b575cc9c69)
    - fuzzer manager for Rust project

