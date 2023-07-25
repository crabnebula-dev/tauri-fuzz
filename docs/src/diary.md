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

### Ideas to explore
- custom wry proxy
    - https://webkitgtk.org/reference/webkit2gtk/2.28.2/WebKitNetworkProxySettings.html
    - not sure you can proxy the custom protocol
- `https://github.com/tauri-apps/tauri-invoke-http` to use http over localhost instead of default Tauri
- types of fuzzer
    - nix/frida/qemu


