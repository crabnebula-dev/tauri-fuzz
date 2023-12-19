# LibAFL

## Flow for our fuzzer

## `Launcher`

Main process:
1. creates one process per core provided
2. starts the restarting manager

Child process:
1. connects to the manager
2. start its `run_client` closure
3. when crashed or when its corpus is used up asks the manager
    for new input from corpus

## `run_client`

- a closure to start/restart a fuzz process
- is provided:
    - shared memory 
    - optionally a state
    - a core id to which the process is associated
- Setup for the process:
    - setup the Frida runtimes
    - setup the observers
    - define feedbacks criteria (if input is interesting)
    - define objective criteria (if input is solution)
    - generate new corpus if necessary
    - setup the scheduler
    - setup the fuzzer 
    - setup the executor
        - inprocess
        - frida executor
    - setup stages
- start the fuzzer with `fuzz_loop`

## `fuzz_loop`

- loop
    1. manager reports progress
    2. `fuzz_one`

## `fuzz_one`

Fuzz a single iteration.
The harness may executed multiple times since an iteration 
consists in proceeding in all the fuzzer stages

1. Get next input from corpus
2. Perform all stages
    1. Tracing with a shadow executor and a cmplog observer
        - `executor::run_target`
        - every comparison operands that were executed are logged
    2. Input2State mutation
        - for n iterations
            - mutate the input to match a comparison operand that was
            logged in previous stage
            - `executor::run_target`
    3. Standard mutation
        - for n iterations
            - mutate the input 
            - `executor::run_target`

## `FridaInProcessExecutor` and `executor::run_target`

Frida executor is a wrapper around `InProcessExecutor`

### on creation

#### `InProcessExecutor`
Setup the process handlers:
- `setup_panic_hook` for crash from the harness
- `crash_handler` for crash in the fuzzing process 
- `timeout_handler` for timeout in the fuzzing process

#### `FridaInProcessExecutor`
Setup the frida runtimes

### run_target

#### `InProcessExecutor`
This is actually not executed from the `FridaInProcessExecutor`.
The `InProcessExecutor` is only called to setup the process handlers on creation
- processor handlers `pre_run_target`
- execute the harness 
- processor handlers `post_run_target`

#### `FridaInProcessExecutor`
- frida runtimes: coverage, drcov, asan, syscallisolation
- for each Frida runtimes
    - `pre_exec`
    - activate the stalker and transform the harness 
    - execute the harness
    - `post_exec`

### when finished or crashed
- If crashed or timeout then call the appropriate handler
- `post_run_reset`


