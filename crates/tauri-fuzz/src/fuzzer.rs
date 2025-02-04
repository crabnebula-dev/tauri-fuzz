// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

//! A libfuzzer-like fuzzer with llmp-multithreading support and restarts

//! The example harness is built for libpng.
// use mimalloc::MiMalloc;
// #[global_allocator]
// static GLOBAL: MiMalloc = MiMalloc;

use frida_gum::{Gum, ModuleMap};
use libafl::generators::{Generator, RandPrintablesGenerator};
use libafl::observers::CanTrack;
use libafl::{
    corpus::{testcase::Testcase, CachedOnDiskCorpus, Corpus, OnDiskCorpus},
    events::{launcher::Launcher, llmp::LlmpRestartingEventManager, EventConfig},
    executors::{inprocess::InProcessExecutor, ExitKind, ShadowExecutor},
    feedback_or, feedback_or_fast,
    feedbacks::{CrashFeedback, MaxMapFeedback, TimeFeedback, TimeoutFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    inputs::BytesInput,
    monitors::MultiMonitor,
    mutators::{
        scheduled::{havoc_mutations, tokens_mutations, StdScheduledMutator},
        token_mutations::I2SRandReplace,
    },
    observers::{HitcountsMapObserver, StdMapObserver, TimeObserver},
    schedulers::{IndexesLenTimeMinimizerScheduler, QueueScheduler},
    stages::{ShadowTracingStage, StdMutationalStage},
    state::{HasCorpus, StdState},
    Error,
};

use libafl_bolts::{
    cli::FuzzerOptions,
    current_nanos,
    rands::StdRand,
    shmem::{ShMemProvider, StdShMemProvider},
    tuples::{tuple_list, Merge},
};

use libafl_frida::helper::FridaRuntime;
use libafl_frida::{
    cmplog_rt::CmpLogRuntime,
    coverage_rt::{CoverageRuntime, MAP_SIZE},
    drcov_rt::DrCovRuntime,
    executor::FridaInProcessExecutor,
    helper::FridaInstrumentationHelper,
};
use libafl_targets::cmplog::CmpLogObserver;
use rangemap::RangeMap;
use std::rc::Rc;
use tauri_fuzz_policies::engine::FuzzPolicy;

use crate::runtime::FunctionListenerRuntime;

/// The main fn, usually parsing parameters, and starting the fuzzer
pub fn fuzz_main<H>(
    harness: H,
    options: &FuzzerOptions,
    tauri_cmd_address: usize,
    policy: FuzzPolicy,
    as_test: bool,
) where
    H: FnMut(&BytesInput) -> ExitKind,
{
    unsafe {
        let res = if as_test {
            fuzz_test(harness, options, tauri_cmd_address, policy)
        } else {
            color_backtrace::install();
            env_logger::init();
            fuzz(harness, options, tauri_cmd_address, policy)
        };
        match res {
            Ok(()) | Err(Error::ShuttingDown) => println!("Finished fuzzing. Good bye."),
            Err(e) => panic!("Error during fuzzing: {e:?}"),
        }
    }
}

/// The actual fuzzer
#[allow(clippy::too_many_lines, clippy::too_many_arguments, dead_code)]
unsafe fn fuzz<H>(
    mut frida_harness: H,
    options: &FuzzerOptions,
    tauri_cmd_address: usize,
    policy: FuzzPolicy,
) -> Result<(), Error>
where
    H: FnMut(&BytesInput) -> ExitKind,
{
    // 'While the stats are state, they are usually used in the broker - which is likely never restarted
    let monitor = MultiMonitor::new(|s| println!("{s}"));

    let shmem_provider = StdShMemProvider::new()?;

    let mut run_client = |state: Option<_>, mgr: LlmpRestartingEventManager<_, _, _>, core_id| {
        // The restarting state will spawn the same process again as child, then restarted it each time it crashes.

        (|state: Option<_>, mut mgr: LlmpRestartingEventManager<_, _, _>, _core_id| {
            let gum = Gum::obtain();

            // Our function listener runtime
            let mut function_listener_rt =
                FunctionListenerRuntime::new(policy.clone(), tauri_cmd_address).unwrap();
            // We init it manually because it may be skipped by libafl_frida if Frida stalker is not enabled
            function_listener_rt.init(&gum, &RangeMap::default(), &Rc::new(ModuleMap::new(&gum)));

            let coverage = CoverageRuntime::new();
            let cmplog = CmpLogRuntime::new();
            let drcov = DrCovRuntime::new();
            let mut frida_helper = FridaInstrumentationHelper::new(
                &gum,
                options,
                tuple_list!(coverage, cmplog, drcov, function_listener_rt),
            );

            // log::info!("Frida helper instantiated: {:#?}", frida_helper);

            // Create an observation channel using the coverage map
            let edges_observer = HitcountsMapObserver::new(StdMapObserver::from_mut_ptr(
                "edges",
                frida_helper.map_mut_ptr().unwrap(),
                MAP_SIZE,
            ))
            .track_indices();

            // Create an observation channel to keep track of the execution time
            let time_observer = TimeObserver::new("time");

            // Feedback to rate the interestingness of an input
            // This one is composed by two Feedbacks in OR
            let mut feedback = feedback_or!(
                // Feedback related to program crash
                CrashFeedback::new(),
                // New maximization map feedback linked to the edges observer and the feedback state
                MaxMapFeedback::new(&edges_observer),
                // Time feedback, this one does not need a feedback state
                TimeFeedback::new(&time_observer)
            );

            let mut objective = feedback_or_fast!(CrashFeedback::new(), TimeoutFeedback::new());

            // If not restarting, create a State from scratch
            let mut corpus_path = options.output.clone();
            corpus_path.pop();
            corpus_path.push("./corpus_discovered");
            let mut state = state.unwrap_or_else(|| {
                StdState::new(
                    // RNG
                    StdRand::with_seed(current_nanos()),
                    // Corpus that will be evolved, we keep it in memory for performance
                    CachedOnDiskCorpus::no_meta(corpus_path, 64).unwrap(),
                    // Corpus in which we store solutions (crashes in this example),
                    // on disk so the user can get them after stopping the fuzzer
                    OnDiskCorpus::new(options.output.clone()).unwrap(),
                    &mut feedback,
                    &mut objective,
                )
                .unwrap()
            });

            // Setup a basic mutator with a mutational stage
            let mutator = StdScheduledMutator::new(havoc_mutations().merge(tokens_mutations()));

            // A minimization+queue policy to get testcasess from the corpus
            let scheduler =
                IndexesLenTimeMinimizerScheduler::new(&edges_observer, QueueScheduler::new());

            // A fuzzer with feedbacks and a corpus scheduler
            let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

            let observers = tuple_list!(edges_observer, time_observer,);

            // Create the executor for an in-process function with just one observer for edge coverage
            let mut executor = FridaInProcessExecutor::new(
                &gum,
                InProcessExecutor::new(
                    &mut frida_harness,
                    observers,
                    &mut fuzzer,
                    &mut state,
                    &mut mgr,
                )?,
                &mut frida_helper,
            );

            // In case the corpus is empty (on first run), reset
            if state.must_load_initial_inputs() {
                if options.input.is_empty() {
                    let mut generator = RandPrintablesGenerator::new(32);
                    let nb_initial_inputs = 8;
                    for _ in 0..nb_initial_inputs {
                        let input = generator
                            .generate(&mut state)
                            .expect("Failed to generate random input");
                        // let testcase = Testcase::<BytesInput>::new(input);
                        let testcase = Testcase::new(input);
                        let _idx = state.corpus_mut().add(testcase)?;
                    }

                // // Force execution of the fuzzer with generated inputs to have initial corpus
                // let _ = state.generate_initial_inputs_forced(
                //     &mut fuzzer,
                //     &mut executor,
                //     &mut generator,
                //     &mut mgr,
                //     8,
                // );
                } else {
                    state
                        .load_initial_inputs(&mut fuzzer, &mut executor, &mut mgr, &options.input)
                        .unwrap_or_else(|_| {
                            panic!("Failed to load initial corpus at {:?}", &options.input)
                        });
                    println!("We imported {} inputs from disk.", state.corpus().count());
                }
            }

            // Create an observation channel using cmplog map
            let cmplog_observer = CmpLogObserver::new("cmplog", true);

            let mut executor = ShadowExecutor::new(executor, tuple_list!(cmplog_observer));

            let tracing = ShadowTracingStage::new(&mut executor);

            // Setup a randomic Input2State stage
            let i2s = StdMutationalStage::new(StdScheduledMutator::new(tuple_list!(
                I2SRandReplace::new()
            )));

            // Setup a basic mutator
            let mutational = StdMutationalStage::new(mutator);

            // The order of the stages matter!
            let mut stages = tuple_list!(tracing, i2s, mutational);

            fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;

            Ok(())
        })(state, mgr, core_id)
    };

    let mut launcher = Launcher::builder()
        .configuration(EventConfig::AlwaysUnique)
        .shmem_provider(shmem_provider)
        .monitor(monitor)
        .run_client(&mut run_client)
        .cores(&options.cores)
        // Builder method stdout_file only in Unix
        // .stdout_file(Some(&options.stdout))
        .broker_port(options.broker_port)
        .remote_broker_addr(options.remote_broker_addr)
        // Store state after crashing, useful if we want to restart the fuzzer at a later time
        // .serialize_state(false)
        .build();

    launcher.launch()
}

/// Fuzz just a single iteration. This is used for testing
/// # Safety
///
/// `frida_gum::Gum::obtain()` is unsafe but the docs does not specify the safety conditions so I
/// don't really know
#[allow(dead_code)]
pub unsafe fn fuzz_test<H>(
    mut frida_harness: H,
    options: &FuzzerOptions,
    tauri_cmd_address: usize,
    policy: FuzzPolicy,
) -> Result<(), Error>
where
    H: FnMut(&BytesInput) -> ExitKind,
{
    let monitor = MultiMonitor::new(|s| println!("{s}"));
    let mut mgr = libafl::events::simple::SimpleEventManager::new(monitor);

    let gum = Gum::obtain();
    let coverage = CoverageRuntime::new();
    let cmplog = CmpLogRuntime::new();
    let mut function_listener_rt =
        FunctionListenerRuntime::new(policy.clone(), tauri_cmd_address).unwrap();
    // We init it manually because it may be skipped by libafl_frida if Frida stalker is not enabled
    function_listener_rt.init(&gum, &RangeMap::default(), &Rc::new(ModuleMap::new(&gum)));

    let mut frida_helper = FridaInstrumentationHelper::new(
        &gum,
        options,
        tuple_list!(coverage, cmplog, function_listener_rt),
    );

    // log::info!("Frida helper instantiated: {:#?}", frida_helper);

    // Create an observation channel using the coverage map
    let edges_observer = HitcountsMapObserver::new(StdMapObserver::from_mut_ptr(
        "edges",
        frida_helper.map_mut_ptr().unwrap(),
        MAP_SIZE,
    ))
    .track_indices();

    // Create an observation channel to keep track of the execution time
    let time_observer = TimeObserver::new("time");

    // Feedback to rate the interestingness of an input
    // This one is composed by two Feedbacks in OR
    let mut feedback = feedback_or!(
        // Feedback related to program crash
        CrashFeedback::new(),
        // New maximization map feedback linked to the edges observer and the feedback state
        MaxMapFeedback::new(&edges_observer),
        // Time feedback, this one does not need a feedback state
        TimeFeedback::new(&time_observer)
    );

    let mut objective = feedback_or_fast!(CrashFeedback::new(), TimeoutFeedback::new());

    // If not restarting, create a State from scratch
    let mut corpus_path = options.output.clone();
    corpus_path.pop();
    corpus_path.push("./corpus_discovered");
    let mut state = StdState::new(
        // RNG
        StdRand::with_seed(current_nanos()),
        // Corpus that will be evolved, we keep it in memory for performance
        CachedOnDiskCorpus::no_meta(corpus_path, 64).unwrap(),
        // Corpus in which we store solutions (crashes in this example),
        // on disk so the user can get them after stopping the fuzzer
        OnDiskCorpus::new(options.output.clone()).unwrap(),
        &mut feedback,
        &mut objective,
    )
    .unwrap();

    // Setup a basic mutator with a mutational stage
    let mutator = StdScheduledMutator::new(havoc_mutations().merge(tokens_mutations()));

    // A minimization+queue policy to get testcasess from the corpus
    let scheduler = IndexesLenTimeMinimizerScheduler::new(&edges_observer, QueueScheduler::new());

    // A fuzzer with feedbacks and a corpus scheduler
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    let observers = tuple_list!(edges_observer, time_observer,);

    // Create the executor for an in-process function with just one observer for edge coverage
    let mut executor = FridaInProcessExecutor::new(
        &gum,
        InProcessExecutor::new(
            &mut frida_harness,
            observers,
            &mut fuzzer,
            &mut state,
            &mut mgr,
        )?,
        &mut frida_helper,
    );

    // In case the corpus is empty (on first run)
    if state.must_load_initial_inputs() {
        if options.input.is_empty() {
            let mut generator = RandPrintablesGenerator::new(32);
            let nb_initial_inputs = 1;
            for _ in 0..nb_initial_inputs {
                let input = generator
                    .generate(&mut state)
                    .expect("Failed to generate random input");
                // let testcase = Testcase::<BytesInput>::new(input);
                let testcase = Testcase::new(input);
                let _idx = state.corpus_mut().add(testcase)?;
            }

            // // Force execution of the fuzzer with generated inputs to have initial corpus
            // let _ = state.generate_initial_inputs_forced(
            //     &mut fuzzer,
            //     &mut executor,
            //     &mut generator,
            //     &mut mgr,
            //     8,
            // );
        } else {
            state
                .load_initial_inputs(&mut fuzzer, &mut executor, &mut mgr, &options.input)
                .unwrap_or_else(|_| {
                    panic!("Failed to load initial corpus at {:?}", &options.input)
                });
            println!("We imported {} inputs from disk.", state.corpus().count());
        }
    }

    // Create an observation channel using cmplog map
    let cmplog_observer = CmpLogObserver::new("cmplog", true);

    let mut executor = ShadowExecutor::new(executor, tuple_list!(cmplog_observer));

    let tracing = ShadowTracingStage::new(&mut executor);

    // Setup a randomic Input2State stage
    let i2s = StdMutationalStage::new(StdScheduledMutator::new(tuple_list!(I2SRandReplace::new())));

    // Setup a basic mutator
    let mutational = StdMutationalStage::new(mutator);

    // The order of the stages matter!
    let mut stages = tuple_list!(tracing, i2s, mutational);

    // fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)?;
    fuzzer.fuzz_one(&mut stages, &mut executor, &mut state, &mut mgr)?;

    Ok(())
}
