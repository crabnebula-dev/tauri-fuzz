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
    executors::{inprocess::InProcessExecutor, ExitKind, TimeoutExecutor},
    feedback_or, feedback_or_fast,
    feedbacks::{CrashFeedback, MaxMapFeedback, TimeFeedback, TimeoutFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    generators::{RandBytesGenerator, RandPrintablesGenerator},
    inputs::{BytesInput, HasBytesVec, HasTargetBytes},
    monitors::{MultiMonitor, SimpleMonitor},
    mutators::scheduled::{havoc_mutations, StdScheduledMutator},
    observers::{HitcountsMapObserver, TimeObserver, VariableMapObserver},
    schedulers::{IndexesLenTimeMinimizerScheduler, QueueScheduler},
    stages::StdMutationalStage,
    state::{HasCorpus, StdState},
    Error,
};

use crate::tauri_fuzz_tools::*;
use crate::utils::*;

pub const MAX_INPUT_SIZE: usize = 1048576; // 1MB

pub fn inprocess_fuzz() {
    // Fuzzed function parameters

    // Hardcoded parameters
    let _timeout = Duration::from_secs(1);
    let broker_port = 1337;
    let cores = Cores::from_cmdline("0").unwrap();
    let _corpus_dirs = [PathBuf::from("./corpus")];
    let objective_dir = PathBuf::from("./crashes");

    let mut harness = |bytes: &BytesInput| {
        let app = setup_tauri_app().expect("Failed to init Tauri app");
        // let app = call_one_tauri_cmd(app, payload_for_tauri_cmd_1(bytes))
        call_one_tauri_cmd(app, payload_for_tauri_cmd_2(bytes));
        ExitKind::Ok
    };

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

        // Create an in-process executor
        let mut executor = InProcessExecutor::new(
            &mut harness,
            // tuple_list!(edges_observer, time_observer),
            (),
            &mut fuzzer,
            &mut state,
            &mut mgr,
        )
        .expect("[fuzzer] Failed to create InProcessExecutor");

        // // Create the executor for an in-process function
        // let mut executor = InProcessExecutor::new(&mut harness, (), &mut fuzzer, &mut state, &mut mgr)
        //     .expect("Failed to create the Executor");

        // // Wrap the executor to keep track of the timeout
        // let mut executor = TimeoutExecutor::new(executor, timeout);

        let mut generator = RandPrintablesGenerator::new(32);
        let _ = state.generate_initial_inputs_forced(
            &mut fuzzer,
            &mut executor,
            &mut generator,
            &mut mgr,
            8,
        );

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
        // .stdout_file(Some("/dev/stdout"))
        .stdout_file(Some("/dev/null"))
        .build()
        .launch()
    {
        Ok(()) => (),
        Err(Error::ShuttingDown) => println!("Fuzzing stopped by user. Good bye."),
        Err(err) => panic!("Failed to run launcher: {err:?}"),
    }
}

#[cfg(feature = "qemu")]
use libafl_qemu::{
    edges::{edges_map_mut_slice, QemuEdgeCoverageHelper, MAX_EDGES_NUM},
    elf::EasyElf,
    emu::Emulator,
    GuestAddr, MmapPerms, QemuExecutor, QemuHooks, Regs,
};

#[cfg(feature = "qemu")]
use crate::qemu::*;

#[cfg(feature = "qemu")]
pub fn qemu_fuzz() {
    // Fuzzed function parameters
    let fuzzed_bynary_path = mini_app_path();
    let fuzzed_func = TAURI_CMD_2;
    let qemu_harness: &QemuHarness = &tauri_cmd_2_harness;

    // Hardcoded parameters
    let _timeout = Duration::from_secs(1);
    let broker_port = 1337;
    let cores = Cores::from_cmdline("0").unwrap();
    let _corpus_dirs = [PathBuf::from("./corpus")];
    let objective_dir = PathBuf::from("./crashes");

    let (emu, fuzzed_func_addr, fuzzed_func_stack_ptr, heap_mem) =
        setup_qemu_emulator(fuzzed_bynary_path, fuzzed_func);

    let mut harness = |input: &BytesInput| {
        qemu_harness(
            &emu,
            input,
            fuzzed_func_addr,
            fuzzed_func_stack_ptr,
            heap_mem,
        )
    };

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

        let mut generator = RandPrintablesGenerator::new(32);
        let _ = state.generate_initial_inputs_forced(
            &mut fuzzer,
            &mut executor,
            &mut generator,
            &mut mgr,
            8,
        );

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
        // .stdout_file(Some("/dev/null"))
        .stdout_file(Some("/dev/stdout"))
        .build()
        .launch()
    {
        Ok(()) => (),
        Err(Error::ShuttingDown) => println!("Fuzzing stopped by user. Good bye."),
        Err(err) => panic!("Failed to run launcher: {err:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Harness to test the fuzzing environment
    fn test_harness(bytes_input: &BytesInput) -> ExitKind {
        let input: u32 = bytes_input_to_u32(bytes_input);

        if input == 4096 {
            println!("[harness] input: {}, bytes_input: {:?}", input, bytes_input);
            ExitKind::Crash
        } else {
            ExitKind::Ok
        }
    }
}
