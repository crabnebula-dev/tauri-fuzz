//! A libfuzzer-like fuzzer using qemu for binary-only coverage
//!

#![allow(unused_imports)]
#![allow(dead_code)]

use core::{ptr::addr_of_mut, time::Duration};
use std::{env, path::PathBuf, process};

use libafl::{
    bolts::{
        core_affinity::Cores,
        current_nanos,
        launcher::Launcher,
        rands::StdRand,
        shmem::{ShMemProvider, StdShMemProvider},
        tuples::tuple_list,
        AsSlice,
    },
    corpus::{Corpus, InMemoryCorpus, OnDiskCorpus},
    events::EventConfig,
    executors::{ExitKind, TimeoutExecutor, inprocess::InProcessExecutor},
    feedback_or, feedback_or_fast,
    feedbacks::{CrashFeedback, MaxMapFeedback, TimeFeedback, TimeoutFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    generators::{RandPrintablesGenerator, RandBytesGenerator},
    inputs::{BytesInput, HasBytesVec, HasTargetBytes},
    monitors::{MultiMonitor, SimpleMonitor},
    mutators::scheduled::{havoc_mutations, StdScheduledMutator},
    observers::{HitcountsMapObserver, TimeObserver, VariableMapObserver},
    schedulers::{IndexesLenTimeMinimizerScheduler, QueueScheduler},
    stages::StdMutationalStage,
    state::{HasCorpus, StdState},
    Error,
};
use libafl_qemu::{
    //asan::QemuAsanHelper,
    edges::{edges_map_mut_slice, QemuEdgeCoverageHelper, MAX_EDGES_NUM},
    elf::EasyElf,
    emu::Emulator,
    GuestAddr,
    //snapshot::QemuSnapshotHelper,
    MmapPerms,
    QemuExecutor,
    QemuHooks,
    Regs,
};

use crate::harness::*;

pub const MAX_INPUT_SIZE: usize = 1048576; // 1MB

// Path to the Tauri sample app binary
// TODO #[no_mangle] does not work with release build
// pub const MINI_APP: &str = "../mini-app/src-tauri/target/release/mini-app";
const MINI_APP: &str = "/f/mini-app/src-tauri/target/debug/mini-app";
const TAURI_CMD_1: &str = "tauri_cmd_1";
const TAURI_CMD_2: &str = "tauri_cmd_2";
pub const CRASH_INPUT_1: &str = "abc";
pub const CRASH_INPUT_2: u32 = 100;

pub fn fuzz() {
    // Get the target file path
    let mut mini_app_path = std::env::current_exe().unwrap();
    mini_app_path.pop();
    mini_app_path.pop();
    mini_app_path.pop();
    mini_app_path.pop();
    mini_app_path.push(String::from("mini-app"));
    mini_app_path.push(String::from("src-tauri"));
    mini_app_path.push(String::from("target"));
    mini_app_path.push(String::from("debug"));
    mini_app_path.push(String::from("mini-app"));

    // Hardcoded parameters
    let _timeout = Duration::from_secs(1);
    let broker_port = 1337;
    let cores = Cores::from_cmdline("0").unwrap();
    let _corpus_dirs = [PathBuf::from("./corpus")];
    let objective_dir = PathBuf::from("./crashes");

    // Initialize QEMU
    env::remove_var("LD_LIBRARY_PATH");
    let mut args: Vec<String> = env::args().collect();
    args.push(mini_app_path.into_os_string().into_string().unwrap());
    let env: Vec<(String, String)> = env::vars().collect();
    let emu = Emulator::new(&args, &env).unwrap();

    let mut elf_buffer = Vec::new();
    let elf = EasyElf::from_file(emu.binary_path(), &mut elf_buffer).unwrap();

    // Get the address of the function `tauri_cmd_1`
    // let fuzzed_func_addr = elf
    //     .resolve_symbol(TAURI_CMD_1, emu.load_addr())
    //     .unwrap_or_else(|| panic!("Symbol \"{}\" not found", TAURI_CMD_1));
    // println!("{} @ {fuzzed_func_addr:#x}", TAURI_CMD_1);


    // Get the address of the function `tauri_cmd_2`
    let fuzzed_func_addr = elf
        .resolve_symbol(TAURI_CMD_2, emu.load_addr())
        .unwrap_or_else(|| panic!("Symbol \"{}\" not found", TAURI_CMD_2));
    println!("[fuzzer] {} @ {fuzzed_func_addr:#x}", TAURI_CMD_2);

    // We run the program until we reach main
    emu.set_breakpoint(fuzzed_func_addr);
    unsafe {
        // TODO
        // Break at main then jump directly to the fuzzed func
        // Tauri did not have time to initialize
        emu.run();
        // println!("Break at {:#x}", emu.read_reg::<_, u64>(Regs::Rip).unwrap());
        // emu.write_reg(Regs::Rip, fuzzed_func_addr)
        //     .unwrap_or_else(|e| panic!("{:?}", e));
        // emu.run();
    };

    println!("[fuzzer] Break at {:#x}", emu.read_reg::<_, u64>(Regs::Rip).unwrap());

    // Get the return address
    let stack_ptr: u64 = emu.read_reg(Regs::Rsp).unwrap();
    let mut ret_addr = [0; 8];
    unsafe { emu.read_mem(stack_ptr, &mut ret_addr) };
    let ret_addr = u64::from_le_bytes(ret_addr);

    println!("[fuzzer] Stack pointer = {stack_ptr:#x}");
    println!("[fuzzer] Return address = {ret_addr:#x}");

    emu.remove_breakpoint(fuzzed_func_addr);
    emu.set_breakpoint(ret_addr);

    // // Reserve some memory in the emulator for dynamic sized data
    // let input_addr = emu
    //     .map_private(0, MAX_INPUT_SIZE, MmapPerms::ReadWrite)
    //     .unwrap();
    // println!("Placing input at {input_addr:#x}");

    // // To test the harness function before the fuzzing loop
    // test_tauri_cmd_2_harness(&emu, fuzzed_func_addr, stack_ptr);

    let mut harness = |input: &BytesInput| tauri_cmd_2_harness(&emu, input, fuzzed_func_addr, stack_ptr);
    // let mut harness = |input: &BytesInput| test_harness(input);

    let mut run_client = |state: Option<_>, mut mgr, _core_id| {
        // Create an observation channel using the coverage map
        // let edges_observer = unsafe {
        //     HitcountsMapObserver::new(VariableMapObserver::from_mut_slice(
        //         "edges",
        //         edges_map_mut_slice(),
        //         addr_of_mut!(MAX_EDGES_NUM),
        //     ))
        // };

        // Create an observation channel to keep track of the execution time
        // let time_observer = TimeObserver::new("time");

        // Feedback to rate the interestingness of an input
        // This one is composed by two Feedbacks in OR
        // let mut feedback = feedback_or!(
        //     // New maximization map feedback linked to the edges observer and the feedback state
        //     MaxMapFeedback::tracking(&edges_observer, true, false),
        //     // Time feedback, this one does not need a feedback state
        //     TimeFeedback::with_observer(&time_observer)
        // );
        let mut feedback = ();

        // A feedback to choose if an input is a solution or not
        // let mut objective = feedback_or_fast!(CrashFeedback::new(), TimeoutFeedback::new());
        let mut objective = CrashFeedback::new();

        // If not restarting, create a State from scratch
        let mut state = state.unwrap_or_else(|| {
            StdState::new(
                // RNG
                StdRand::with_seed(current_nanos()),
                // Corpus that will be evolved, we keep it in memory for performance
                InMemoryCorpus::new(),
                // Corpus in which we store solutions (crashes in this example),
                // on disk so the user can get them after stopping the fuzzer
                OnDiskCorpus::new(objective_dir.clone()).unwrap(),
                // States of the feedbacks.
                // The feedbacks can report the data that should persist in the State.
                &mut feedback,
                // Same for objective feedbacks
                &mut objective,
            )
            .unwrap()
        });

        // A minimization+queue policy to get testcasess from the corpus
        // let scheduler = IndexesLenTimeMinimizerScheduler::new(QueueScheduler::new());
        let scheduler = QueueScheduler::new();

        // A fuzzer with feedbacks and a corpus scheduler
        let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

        // let mut hooks = QemuHooks::new(&emu, tuple_list!(QemuEdgeCoverageHelper::default()));
        let mut hooks = QemuHooks::new(&emu, ());

        // Create a QEMU in-process executor
        let mut executor = QemuExecutor::new(
            &mut hooks,
            &mut harness,
            // tuple_list!(edges_observer, time_observer),
            (),
            &mut fuzzer,
            &mut state,
            &mut mgr,
        )
        .expect("[fuzzer] Failed to create QemuExecutor");


        // // Create the executor for an in-process function
        // let mut executor = InProcessExecutor::new(&mut harness, (), &mut fuzzer, &mut state, &mut mgr)
        //     .expect("Failed to create the Executor");



        // // Wrap the executor to keep track of the timeout
        // let mut executor = TimeoutExecutor::new(executor, timeout);

        let mut generator = RandPrintablesGenerator::new(4);
        let _ = state.generate_initial_inputs_forced(&mut fuzzer, &mut executor, &mut generator, &mut mgr, 8);


        // Setup an havoc mutator with a mutational stage
        let mutator = StdScheduledMutator::new(havoc_mutations());
        let mut stages = tuple_list!(StdMutationalStage::new(mutator));

        fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;
        Ok(())
    };

    // The shared memory allocator
    let shmem_provider = StdShMemProvider::new().expect("Failed to init shared memory");

    // The stats reporter for the broker
    // let monitor = MultiMonitor::new(|s| println!("{s}"));
    let monitor = SimpleMonitor::new(|s| println!("[fuzzer] {s}"));

    // Build and run a Launcher
    match Launcher::builder()
        .shmem_provider(shmem_provider)
        .broker_port(broker_port)
        .configuration(EventConfig::from_build_id())
        .monitor(monitor)
        .run_client(&mut run_client)
        .cores(&cores)
        .stdout_file(Some("/dev/stdout"))
        .build()
        .launch()
    {
        Ok(()) => (),
        Err(Error::ShuttingDown) => println!("Fuzzing stopped by user. Good bye."),
        Err(err) => panic!("Failed to run launcher: {err:?}"),
    }
}


