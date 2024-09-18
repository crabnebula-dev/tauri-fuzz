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

- try to trigger a command programmatically
- `PendingWindow` has fields
  - `js_event_listeners`
  - `ipc_handler`
- Check `wry` crate
  - webview sends message to the Rust host using `window.ipc.postMessage("message")`
- Try to capture IPC using wireshark
  - listening on the loopback interface
  - did not work, certainly **tauri does not use internet socket**
- Try to capture IPC using `strace`
  - we see traces of `recvmsg` and `sendmsg` syscalls
  - using `ss -pO | grep mini/WebKit` we see existences of open sockets for these processes
  - Unix sockets can be tracked using this [sockdump](https://github.com/mechpen/sockdump)
    - `sockdump` can output to pcap format that is readable by wireshark

## 3

- Trying to `sockdump` the mini-app sockets
  - checking sockets file in `/proc/$PID/fd`
  - `lsof -p $PID` lists open files for a process
  - **tauri command does not seem to pass through unix sockets**
    - `ss` show that the open sockets have no data going through them
    - this is confirmed using `sockdump`
- Checking `tauri`, `wry` and `tao` code to see where IPC comes from
  - connect to local version of wry and tauri
  - `tao::EventLoop::run_return` when spawning x11 thread contains
    `let (device_tx, device_rx) = glib::MainContext::channel(glib::Priority::default());`

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

- x86_64 calling convention checked
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
  - then sending another message which is **not** `ExitRequestedEventAction::Prevent`

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
  1.  `cargo fuzz run fuzz_target`
  2.  `cargo fuzz coverage fuzz_target`
  3.  Show code coverage with `llvm-cov show` > `llvm-cov show \
-instr-profile=coverage/fuzz_target_1/coverage.profdata \
-Xdemangler=rustfilt target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/fuzz_target_1 \
-use-color --ignore-filename-regex='/.cargo/registry' \
-output-dir=coverage/fuzz_target_1/report -format=html \
-show-line-counts-or-regions \
-ignore-filename-regex='/rustc/.+'` - docs on [https://llvm.org/docs/CommandGuide/llvm-cov.html#llvm-cov-show](https://llvm.org/docs/CommandGuide/llvm-cov.html#llvm-cov-show) - bin with coverage information is generated at `target/arch_triple/coverage/arch_triple/release/fuzz_target` - profile file is generated at `coverage/fuzz_target/coverage.profdata`
  4.  Create a summary report with `llvm-cov report` > `llvm-cov report \
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

Frida is a binary analyser with 2 main features - _Stalker_ code-tracing engine - follow threads and trace every instruction
that are being called - uses a technique called _dynamic recompilation_ - while a program is running the current basic block is copied and stored in caches - these copy can be modified and executed on demand - the original instructions are unmodified - _Interceptor_ hooks - allows inspection and modification of the flow of function calls - different possible techniques but most common are trampoline based hooks - code is inserted at the beginning of a function A to execute another function B so function B is "inserted" in the middle of function A

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

- SET a flag every time you change of running environment (disable flag when running fuzzer code)
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

## 22

- Update docs on syscalls
- Compile `mini-app` as a dylib
  - libafl prevent instrumenting its own crate to prevent weird recursion
- Clean the `mini-app` fuzzing code
- Make `mini-app` dynamic
  - to use the binary directly and linking with the dynamic `libmini_app.so`
  - LD_LIBRARY_PATH='/home/user/tauri-fuzzer/mini-app/src-tauri/fuzz/target/debug/deps:/home/user/.rustup/toolchains/1.70-x86_64-unknown-linux-gnu/lib:/home/user/tauri-fuzzer/mini-app/src-tauri/fuzz/target/debug:/home/user/.rustup/toolchains/1.70-x86_64-unknown-linux-gnu/lib'
- Create a tauri command that do a system call without using the libc

## 23

- Create a separate crate `tauri_fuzz_tools` for helper functions
  - this function connect Tauri to LibAFL
- Change whole repo to a workspace
- Catch a call to libc
  - Check any "calls" and destination address
    - we don't need to instrument libc
    - we may miss hidden calls
  - Instrument the libc and verify the instruction location
    - we need to instrument libc and all libc instructions will be analysed
    - easier to implement
- Found how to get libc symbols through `friga_gum::Module::enumerate_exports`
- Strange "double crash bug"
  - does not appear when removing coverage from the runtimes

## 24

- Inspect minimization
  - misunderstanding of what minimization is
  - thought that minimization would reduce the number of solutions found to only keep ones with different coverage
  - Real use of minimization:
    - reduce size of the "interesting" inputs while preserving the code coverage
    - removes the "noise" in inputs for easier analysis and mutations
  - Docs and examples can be found at:
    - https://docs.rs/libafl/latest/libafl/corpus/minimizer/trait.CorpusMinimizer.html
    - an example fuzzer in: "LibAFL/fuzzers/libfuzzer_libpng_cmin/src/lib.rs"

## 25

- on Windows
  - change visibility for the different modules
  - make sure that given paths are portable
- Noticed that when opening a file `fopen` is not called but `open`
- Another issue is that interceptor do not distinguish between calls from the crates and the code we are targeting
  - we need to have an interceptor that sets up a flag on the tauri command we are fuzzing (then it's single threaded?)

## 26

- Trying to setup the interceptor only when the harness functions are entered
  - when entering the tauri command we are fuzzing
  - when we are entering the harness: `setup_tauri_mock` + `on_msg`
- In our mental model it's one thread per harness executed
  - the `SyscallIsolationRuntime` is initiated for each thread
  - we should be able to have one flag per `SyscallIsolationRuntime` to setup when the harness function has been entered
- Bug but maybe disable other runtime

## 27

- Finding function symbol in the runtime with a pointer rather than a name
  - name mangling make it harder
  - more precise
- the fuzzer intercepts the `open` syscall
  - this happens in the fuzzer `panic_hook` to write state to disk
    - it's difficult to set the `SyscallIsolationRuntime` flag from the `panic_hook`
    - we dodge the issue by rewriting the `panic_hook`
  - this happens with the stalker

## 28

- Trying to refact `fuzzer.rs` to have the same code to use `fuzz_one` or `Launcher`
  - really difficult due to the numerous traits used by LibAFL
  - the trick they use is to use a closure so we don't need to precise a type for all objects used
  - but to turn this into a function
    - using `impl` return type does not work due to Rust not supporting nested `impl`
    - returning generic type not really working either since the return type is clearly defined in the function body
    - using exact type is super difficult too due to the complexity of the types in LibAFL
  - I think I need a rust expert for this
- Writing tests for our fuzz targets
  - Issue is that tests that crash actually are handled by the fuzzer and actually `libc::_exit(134)`
  - This is not handled by cargo tests
  - What I've tried
    - `#[should_panic]` this is not a panic so it does not work
    - `panic::setup_hook(panic!)` this is rewritten by the fuzzer =(
    - `uses abort rather than panic` does not work either
  - Solved by wrapping the test in another process and using and self calling the binary
    with `Command::new(std::env::current_exe())`

## 29

- Working on fuzzing policy
  - Need a more generic and flexible way to give a security policy, need the security team for their inputs
  - security policies should be provided as constants for performance
- Restructure the project
  - fuzzer and security policy code moved to the application being fuzzed `fuzz` directory
  - user can now directly see the fuzzer and the policy used rather than looking at external crate
- Another race condition happened
  - **be sure to drop the harness flag before calling any function that might panic**
- For conditions we're currently using functions rather than closures
  - this is to avoid any rust issue with trait object
  - this should be improved in the future

## Call Tauri inbuilt command such as fs_readFile

- Improve `create_invoke_payload`
  - allow to have an argument specifying a module
  - distinguish between an invocation between a custom command and an inbuilt one
- These commands requires a shared state to be managed by the Tauri mock runtime
  - error message triggered is `state() called before manage() for given type`
  - we can't use our helper function `mock_builder_minimal`
  - use `mock_builder` instead
- The `InvokeRequest` looks like

```rust,ignore
InvokeRequest {
    cmd: "plugin:fs|read_file",
    callback: CallbackFn(
        2482586317,
    ),
    error: CallbackFn(
        1629968881,
    ),
    url: Url {
        scheme: "http",
        cannot_be_a_base: false,
        username: "",
        password: None,
        host: Some(
            Ipv4(
                127.0.0.1,
            ),
        ),
        port: Some(
            1430,
        ),
        path: "/",
        query: None,
        fragment: None,
    },
    body: Json(
        Object {
            "options": Object {
                "dir": String("toto"),
            },
            "path": String("foo.txt"),
        },
    ),
    headers: {
        "content-type": "application/json",
        "origin": "http://127.0.0.1:1430",
        "referer": "http://127.0.0.1:1430/",
        "accept": "*/*",
        "user-agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15",
        "tauri-callback": "2482586317",
        "tauri-error": "1629968881",
        "tauri-invoke-key": "[Ic/:jX^L^q#hDgJd7)U",
    },
    invoke_key: "[Ic/:jX^L^q#hDgJd7)U",
}
```

- Don't forget to configure the `allowlist` to allow the scope

## 30

- Move `mini-app/src-tauri/fuzz/` to `mini-app-fuzz`
  - seamless transition, just had to change dependency in the workspace `Cargo.toml`
- Writing a presentation with Reveal.js
  - presentation added to the mdbook
- Bump to Rust version 1.76
  - Update VM to Rust 1.70 -> 1.76
  - Unroll the `clap` package version in `libafl_bolts`: `~4.4` -> `4.0 (4.5)`
    - We pinned it because it was not compatible with last version of Rust I was using
- Make `LibAFL` a submodule
  - `LibAFL` is also a Rust workspace itself so we had to `exclude = ["LibAFl"]` it from the root `Cargo.toml`
  - `git config submodule.recurse = true` do not seem to work to pull recursively the last LibAFL commit
- Writing user guide to

## 31

- Restructure the repo with a classical monorepo architecture
  - `docs/` with mdbook and slides
  - `examples/` with mini-app and its fuzz code
  - `crates/` with `LibAFL`, `policies`, `fuzzer`
- Create a TOML configuration file for the fuzzer
  - more simple intermediary type to `libafl_bolts::FuzzerOptions`
- Why is our code coverage not working for the moment?
  - the `harness` and `libs_to_instrument` options were empty meaning the stalker was not applied on any part of executable
  - `cmplog` module is not implemented for x86-64
  - even when adding the executable to `harness`, it is removed by `libafl_frida` to avoid the stalker from analyzing its own code and get recursion
    - this is annoying with rust where you usually use static libraries so you get one big executable
    - a solution would be to make LibAFL a dynamic lib
    - with a quick try without persevering we get some link errors
      - this is not a mess I want to invest time in currently
    - another solution would be to be able to give exact memory ranges that we want frida stalker to work on
      - currently the precision is per `Module`
      - a module is more a less a library
      - for Rust it signifies the whole executable with all its crates + basic C libraries
      - ideally we would have the stalker on the main binary and not on any of its crate
      - We could make a PR for that
- When running our binaries the fuzz_solutions are written in the wrong directory
  - `cargo test` executes in the root directory of the crate containing the tests
  - `cargo run` takes current directory where command is executed as root directory

## Porting to 2.0

- InvokeRequest new format

```rust,ignore
#### Template for a plugin InvokeRequest
InvokeRequest {
    cmd: "plugin:fs|read_file",
    callback: CallbackFn(
        3255320200,
    ),
    error: CallbackFn(
        3097067861,
    ),
    url: Url {
        scheme: "http",
        cannot_be_a_base: false,
        username: "",
        password: None,
        host: Some(
            Ipv4(
                127.0.0.1,
            ),
        ),
        port: Some(
            1430,
        ),
        path: "/",
        query: None,
        fragment: None,
    },
    body: Json(
        Object {
            "options": Object {},
            "path": String("README.md"),
        },
    ),
    headers: {},
}
```

- Calling plugin commands with the `MockRuntime` (such as `fs:readFile`)
  - Scope can be modified programmatically using

```rust,ignore
  let scope = app.fs_scope();
  scope.allow_file("/home/adang/boum/playground/rust/tauri2/src-tauri/assets/foo.txt");
```

- `RuntimeAuthority` requires an acl and resolved acl
  - the `RuntimeAuthority.acl`
    - isn't modifiable programmatically
    - defines which permissions are allowed to be used by the application capabilities
    - ACL from the runtime authority is generated at buildtime in the `Context`
    - code generation to get the Tauri app context is located at `tauri-codegen::context::context_codegen`
  - `Resolved`
    - commands that are allowed/denied
    - scopes associated to these commands
    - it is initialized from the complete acl and the capabilities declared by the application
- When building a Tauri v2 app `tauri-build` :
  - path to permission manifests from each plugin are stored in environment variables
    - 3 env variables per plugin used
      - `DEP_TAURI_PLUGIN_FS_PERMISSION_FILES_PATH`
        - where the permissions declaration for this plugin are declared
      - `DEP_TAURI_PLUGIN_FS_GLOBAL_API_SCRIPT_PATH`
        - JS script containing the API to call commands from the plugin
        - I think this is only used when the option `withGlobalTauri` is set
      - `DEP_TAURI_PLUGIN_FS_GLOBAL_SCOPE_SCHEMA_PATH`
        - schema for the scopes of the plugin
  - the permissions manifests are parsed
    - manifests contain all the permissions declared by plugins
  - parse the capabilities file
  - check that declared capabilities are compatible with information given by the manifests
- InvokeRequest `url`
  - to have request that are deemed `Local` use `tauri://localhost`
- Fuzzer does not need to `tauri_app_builder.run(...)` just if
  - we don't need an event loop
  - we don't need to setup the app
- we don't need to interact with the app state
- Url for `InvokeRequest` for local tauri commands is
  - "http://tauri.localhost" for windows and android
  - "tauri://localhost" for the rest

## 32

- Github actions
  - use act to run github actions locally
  - to run test as github actions locally
  - with linux container: `act -W ".github/workflows/build_and_test.yml" -j Build-and-test-Fuzzer -P ubuntu-latest=catthehacker/ubuntu:act-latest`
  - on windows host: `act -W ".github/workflows/build_and_test.yml" -j Build-and-test-Fuzzer -P windows-latest=self-hosted --pull=false`
  - always do the command twice, the first one usually fails for unknown reasons
- Bug with Rust 1.78
  - Rust 1.78 enables debug assertions in std by default
  - `slice::from_raw_parts` panics when given a pointer which is not aligned/null/bigger than `isize::max`
  - Bug in libafl_frida which trigger this situation when
    - `stalker_is_enabled` is set to true in `libafl_frida/src/helper.rs`
    - and no module is specified to be stalked
    - as a reminder stalker is enabled if we want to use the code coverage
  - Bug for coverage when stalker is enabled
    - in `libafl_frida/src/helper.rs::FridaInstrumentationHelperBuilder::build`
    - the `instrument_module_predicate` return true for the harness
    - but the `ModuleMap` returned by `gum_sys` is empty
    - this provokes a panic from Rust 1.78
    - current fix is to disable coverage but not good enough

## 33

- Generating test for cli
  - issue killing the fuzzer process after launching it with cli
  - how do we get the pid of the fuzzer process which is a different process from the binary ran by `cargo run`
  - rust does not have command with timeout
  - We do it by querying the system for process with certain exact name
    - this is not super robust
    - behaviour is also platform dependent
    - we limit this test to linux platform to avoid future complications
- New issue introduced with Tauri `2.0.0-beta.22`

  - `fs::read_file` returns `InvokeBody::Raw(Vec<u8>)`
  - to get Rust type from this raw value, Tauri provides this function

  ```rust, ignore
  pub fn deserialize<T: DeserializeOwned>(self) -> serde_json::Result<T> {
    match self {
        ...
        InvokeBody::Raw(v) => serde_json::from_slice(&v),
    }
  }
  ```

  - this is flawed as `serde_json::from_slice(&v)` expects `v` to be `bytes of JSON text` (from `serde_json` documentation)
  - what was given from `fs::read_file` are raw bytes of the content of a file and this triggers a serialization error
  - for the function `deserialize` to work we need an additional conversion of the raw bytes into bytes of json text
  - a proposal that does not completely fix the issue but at least allow us to recuperate a `Vec<u8>` that can be used for further conversion:

  ```rust, ignore
  pub fn deserialize<T: DeserializeOwned>(self) -> serde_json::Result<T> {
    match self {
      ...
      InvokeBody::Raw(v) => {
        let json_string = serde_json::to_string(&v).unwrap();
        serde_json::from_slice(&json_string.into_bytes())
      }
    }
  }
  ```

  - either the function `deserialize` in Tauri is wrong or what is returned from `fs::read_file` is wrong

## Windows

### Issues

#### Troubles building fuzzer for windows with LibAFL

- execution issue which does not appear when commenting calls to the LibAFL fuzzer
- using the `msvc` toolchain
  - building works fine
  - we get `(exit code: 0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND)` when running the binary
  - this happens when windows fails to load a dll
    - [dependencywalker](https://www.dependencywalker.com/) to investigate can help but now is deprecated
    - make sure that there is no discrepancy between loader version and compilation toolchain
- using the `windows-gnu` toolchain
  - I need to install `gcc` for linking
- what toolchain should I use?
  - depends on which dynamic library I need to link to
  - look into libafl repo for hints
  - in github action we see that they use the windows default stable toolchain
    - that should be `msvc`
- Error found `TaskEntryDialog` entrypoint could not be found
  - running the fuzzer from windows ui
    - or using [cbc](https://ten0s.github.io/blog/2022/07/01/debugging-dll-loading-errors)
  - Dependency walker shows the missing modules
    - one of the main missing module is `API-MS-WIN-CORE`
  - Using `ProcessMonitor` with a filter on `tauri_cmd_1.exe`
    - run the executable and you get all the related events
- Big chances it is related to `tauri-build` which does a lot in windows
  - reintroduce a `build.rs` file with `tauri_build::build()`
  - Find a way to have a generic and minimal `tauri.conf.json` for the fuzz directory

#### `frida_gum` does not find any symbol or export in the Rust binary

- check symbols manually with equivalent of `nm` which is `dumpbin.exe`
  - use developer terminal to use `dumpbin.exe` easily
  - Windows executables are stripped of any export symbols
- Our previous approach used debug symbols to find the native pointer to the harness
  - debug symbols are not available on windows (in the file directly but separate ".pdb" file)
  - We change so we use the raw address provided at the beginning to create the `NativePointer`

#### No display from crash result

- When running the fuzzer the crash happens but nothing is displayed
- We change the panic hook order such that original panic hook is executed before the fuzzer panic hook

#### Error status of crashed program in fuzzer

- In windows the error status chosen by LibAFL is 1 instead of 134

#### Find equivalent of libc functions

- Example with finding a CRT function that is used to open a file
- Debug a function that is opening a file with Visual Studio and tracks the execution
  - fs.rs file needs to be provided.
    - It's in `C:\Users\alex-cn\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\std\src\sys\windows\fs.rs`
  - Find the function called `c::CreateFileW` used t
  - in the `c` directory find that `CreateFileW` comes from the `kernel32` dll
- Check Rust source code and finds the OS-specific implementation

#### Tests show `tauri_fuzz_tools-917323a62e294d07.exe write_foo (exit code: 0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND)`

- This is similar error message to previous issue which was missing `tauri_build::build()`
  - checked that build script is executed to build the tests
  - issue seems to come from the `tauri-fuzz-tools` crate
- From experiments `tauri_fuzz_tools` tests
  - fails to run from workspace directory with `cargo t`
    - executable produced is bigger than the successful one
  - run fine from workspace directory with `cargo t -p tauri_fuzz_tools`
    - executable produced is smaller than the failing one
  - run fine when executing `cargo t` from the crate directory
  - runs fine when putting `tauri_fuzz_tools` as the sole default member of the workspace
  - fails when putting `tauri_fuzz_tools` as default member with any other member
- Adding a Windows Manifest file works to remove the error message
  - https://github.com/tauri-apps/tauri/pull/4383/files
  - Does not explain why the compilation worked in certain cases but not in other =(
- Tried with crate `embed-manifest`
  - crate seems outdated contain build instruction not recognized

#### Fetching values from register does not give expected value

- the policy "block_file_by_names" does not work
- Windows do not use utf-8 encoding but utf-16 for strings
  - use the `windows` crate to import correct windows type and do type conversion

#### Conflicting C runtime library during linking

```ignore
= note: LINK : warning LNK4098: defaultlib "LIBCMT" conflicts with use of other libs; use /NODEFAULTLIB:library
          LINK : error LNK1218: warning treated as error; no output file generated
```

- This seems to happen
- I don't really know what made this bug appear
  - one suspicion is the upgrade to Rust 1.78
  - Amr had this first and I only got it when I manually updated my `rustup`
- Cause of the event
  - conflicting lib c runtime have been found
  - I see in the compilation logs that we already link against the "msvcrt.lib" c runtime
  - my guess is that some library is trying to link against "libcmt" on top
- Solution found
  - linker options added in `.cargo/config.toml` file
  ```ignore
  [target.x86_64-pc-windows-msvc]
  rustflags = ["-C", "link-args=/NODEFAULTLIB:libcmt.lib"]
  ```
- to be honest I don't really understand what's happening precisely and I don't want to dig further.
  But I'm happy to have found a solution quickly but I expect this to bite me back in the future

#### NtCreateFile use flags different from the doc

- doc: https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntcreatefile
- from the doc `NtCreateFile` is supposed to use flags such as:
  - FILE_GENERIC_READ: 0x00120089
  - FILE_GENERIC_WRITE: 0x00120116
  - FILE_READ_DATA: 0x00000001
- from the experimentations we get values such as:
  - open file in read only: 0x80100080
  - open file in write only: 0x40100080
- this matches other known windows constants that exist are:
  - GENERIC_READ: 0x80000000
  - GENERIC_WRITE: 0x40000000
- we will use these flags eventhough this is different from what described from the doc

### Docker on Windows

- Docker daemon can be started by launching Docker desktop
- _docker-credential-desktop not installed or not available in PATH_
  - in the file `C:\Users\user\.docker\config.json`
  - delete `credsStore` field

### Tools for debugging

- `ProcessMonitor` to see all the events related to a process
- `DependencyWalker` to investigate issue related to modules/dlls

### Default policy

- We want to have a default policy that catches any calls to an external binary that returns an error
  - our intuition is a call to an external binary that can result into a syntax error
    also has a chance to be vulnerable to an exploit
  - with the fuzzer there is a high chance "vulnerable" calls to external process will result in syntax error
- We want to attach to Rust `std::process::Command::spawn/output`
  - I don't see the symbol of these functions in the binary, I don't really get why
- Maybe the solution is to attach to `execv` family of calls and monitor the return status of the call
  - this is lower level that rust `Command`, we can catch more external interactions from the app we monitor
  - I believe this is called by rust `Command` but I need to check that
- All functions from `exec` family calls `execve`
  - from this implementation of libc [https://github.com/zerovm/glibc/blob/master/posix/execv.c](https://github.com/zerovm/glibc/blob/master/posix/execv.c)
- Fuzzer crashes when monitoring `execv`

  - it does not crash when monitoring other functions
  - it crashes in the fuzzer code
    - with fuzz_test
      - with a rule that never blocks
        - it crashes in the harness and is captured by the fuzzer
      - with a rule that always blocks
        - it crashes in the harness too
        - actually the harness has time to finish, corruption appears after the harness
        - `*** stack smashing detected ***: terminated`
    - with fuzz_main
      - with a rule that always blocks
        - it crashes in the harness when the tauri command is finished but the harness has not finished yet
        - `*** stack smashing detected ***: terminated`
      - with a rule that never blocks
        - it crashes in the harness when the tauri command is finished but the harness has not finished yet
        - `*** stack smashing detected ***: terminated`
  - I think that after the harness the fuzzer calls execve before the flag is removed
  - Call order starting from when the harness is being called
    - in `libafl::Executor::run_target`: `let ret = (self.harness_fn.borrow_mut())(input);`
    - `libafl::executors::inprocess::GenericInProcessExecutor`
    - `core::ops::function::FnMut::call_mut`
    - `ls_with_rust::harness` with `ls_with_rust` the binary being executed
      - `_gum_function_context_begin_invocation`
        - `gum_tls_key_get_value`
        - `pthread_getspecific`
        - `gum_tls_key_set_value`
        - `get_interceptor_thread_context`
        - `gum_thread_get_system_error`
        - `gum_invocation_stack_push`
          - `gum_sign_code_pointer`
        - `gum_rust_invocation_listener_on_enter`
          - `frida_gum::interceptor::invocation_listener::call_on_enter`
            - `libafl_frida::syscall_isolation_rt::HarnessListener::on_enter`
        - `gum_thread_set_system_error`
          - `__errno_location@plt`
        - `gum_tls_key_set_value`
        - `pthread_setspecific`
      - harness code ...
    - pure asm code that push registers on the stack
      - that looks like context switch with context being saved on the stack
    - `_gum_function_context_end_invocation`
      - `gum_tls_key_set_value`
        - `pthread_setspecific@plt`
      - `gum_thread_get_system_error`
        - `__errno_location@plt`
      - `get_interceptor_thread_context`
        - `_frida_g_private_get`
          - `g_private_get_impl`
          - `pthread_getspecific@plt`
      - `gum_sign_code_pointer`
      - `gum_rust_invocation_listener_on_leave`
        - `frida_gum::interceptor::invocation_listener::call_on_leave`
          - `frida_gum::interceptor::invocation_listener::InvocationContext::from_raw`
          - `libafl_frida::syscall_isolation_rt::HarnessListener::on_leave`
      - `gum_thread_set_system_error`
        - `_errno_location@plt`
      - `_frida_g_array_set_size`
      - `gum_tls_key_set_value`
        - `pthread_setspecific`
    - pure asm code that pop stack values into registers
      - restore context switch
  - `__execvpe_common.isra`: here we crash

- Correct execution trace at the end of the harness:
  - pure asm code that push registers on the stack
    - that looks like context switch with context being saved on the stack
  - `_gum_function_context_end_invocation`
    - `gum_tls_key_set_value`
      - `pthread_setspecific@plt`
    - `gum_thread_get_system_error`
      - `__errno_location@plt`
    - `get_interceptor_thread_context`
      - `_frida_g_private_get`
        - `g_private_get_impl`
        - `pthread_getspecific@plt`
    - `gum_sign_code_pointer`
    - `gum_rust_invocation_listener_on_leave`
      - `frida_gum::interceptor::invocation_listener::call_on_leave`
        - `frida_gum::interceptor::invocation_listener::InvocationContext::from_raw`
        - `libafl_frida::syscall_isolation_rt::HarnessListener::on_leave`
    - `gum_thread_set_system_error`
      - `_errno_location@plt`
    - `_frida_g_array_set_size`
    - `gum_tls_key_set_value`
      - `pthread_setspecific`
  - pure asm code that pop stack values into registers
  - here we don't crash contrary to above
- New approach where we detach the frida listeners of monitored functions instead of deactivating them
  - Contrary to what the docs says, calling `Gum::obtain` produce a deadlock (in doc it's supposed to do a no-op)
  - Without `Gum::obtain` we can't detach the monitored function listeners
- Weirdest thing ever: the crash does not appear anymore with gdb when putting a breakpoint on `execve`
- I'm temporarily giving up on monitoring `execv`
  - I still think it's the
- Trying with `__execve` instead of `execve`
  - maybe C weak links mess up with Frida
  - not working either
- Ok I just notice that my approach was wrong anyway
  - `execve` usually called in the child process after being forked
  - Frida rust bindings do not support monitoring the child process anyway
  - I still don't know why there was a bug

### Improving engine code

- Our rules now use `Fn` closure trait rather than `fn` object
- this allow us to make rules that are more flexible with captured variables and arguments
- the main issue was to use `Box<dyn Fn>>` that were also implementing `Clone`
  - inspiration from Tauri code with all the callbacks
  - this thread helped us solve the issue: https://users.rust-lang.org/t/how-to-clone-a-boxed-closure/31035/7
  - replace `Box` by `Arc`
  - we could also create manual cloneable `Box<dyn Fn>>` like this example
    - https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=6ca48c4cff92370c907ecf4c548ee33c

### Improve tests

- Refactor tests to avoid too much repetition
- All tests are gathered in a single crate to avoid too much disk usage

### Default policy

- We have a new approach where we monitor the `std::process::Command` API

  - we detect any new creation process
    - we track `Command::status`, `Command::output`, `Command::spawn`
    - ideally we could track a single function: `std::sys::windows/unix::process::Command`
      - all the above functions call this private function
      - unfortunately this function is private and we can't attach to it with Frida
      - actually it seems we can! Just found this in the symbols
      ```ignore
      "SymbolDetails _ZN3std3sys3pal4unix7process13process_inner71_$LT$impl$u20$std..sys..pal..unix..process..process_common..Command$GT$5spawn17hffc9080bc0517252E: [0x0000555740c67360, 0x0000555740c680c1]",
      ```
  - we can also detect error status of external process
    - we track `Command::output`, `Command::status`, `Child::wait`, `Child::try_wait`, `Child::wait_with_output`,
    - an issue is that we don't know from which binary we returned from
  - Limit of our current approach is that we can only detect invocation of external binaries from the Rust API
    - we don't detect invocation of ext binaries through libc `fork` + `execve`
    - but we could monitor `wait` and `waitpid` to track error status

- We monitor `wait` and `waitpid`
  - this is a superset of monitoring rust `std::process::Command`
  - we had to modify the policy engine to add a storage that can store function parameter at entry
    that can then be reused when analysing the function at exit
    - this is necessary due to common C pattern that store results of a function in a mutable pointer given as parameter
  - Question: Do libc usually call `wait` or `waitpid` after spawning a child process?
    - they should otherwise it would create zombie process
  - Can we do better?
    - ideally we would track `fork + execve` but it seems too complex with Frida
    - external process can be called by other means than creating a child process
      - for example in SQL an RPC is used to talk to SQL server and no `fork` is ever used
      - we also need to track networking then =(
    - we are using the assumption that a child process will return 0 as exit status when the execution
      went well. Is it always true?

#### libc wait

- we want to also capture error status of child processes that were invoked through the libc API
  - from my knowledge these child processes are invoked using `fork` then `execve`
  - one way to get the return status of these child processes is to capture calls to `wait` from the parent process
- the issue with `wait` is that the child exit status is returned through mutating a variable that was sent as argument
  and not through the return value
- to fix that we may need to store the pointer that was provided as argument to be able to check it on exit
  - we implemented that and it works great

### Bug with plugin fs_readFile

- For unknown reason when accessing the filesystem with `tauri_plugin_fs` the interception does not occur
  - this does not occur when accessing the filesystem with other functions
- Possible reasons for that:
  - `tauri_plugin_fs::read_file` does not call `open`
    - this is unlikely since `tauri_plugin_fs` uses this Rust code
      `let file = std::fs::OpenOptions::from(open_options.options)`
  - Tauri plugins are executed in a context which are not tracked by Frida
    - In another process?
    - Let's check the Tauri changelog
- We solve this in another PR
