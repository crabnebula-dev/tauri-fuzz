//! A libfuzzer-like fuzzer with llmp-multithreading support and restarts

//! The example harness is built for libpng.
// use mimalloc::MiMalloc;
// #[global_allocator]
// static GLOBAL: MiMalloc = MiMalloc;

use frida_gum::Gum;
use libafl::generators::{Generator, RandPrintablesGenerator};
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

use libafl_frida::{
    cmplog_rt::CmpLogRuntime,
    drcov_rt::DrCovRuntime,
    coverage_rt::{CoverageRuntime, MAP_SIZE},
    executor::FridaInProcessExecutor,
    helper::FridaInstrumentationHelper,
    syscall_isolation_rt::SyscallIsolationRuntime,
};
use libafl_targets::cmplog::CmpLogObserver;
use policies::engine::FuzzPolicy;

/// The main fn, usually parsing parameters, and starting the fuzzer
pub fn fuzz_main<H>(
    harness: H,
    options: FuzzerOptions,
    tauri_cmd_address: usize,
    policy: FuzzPolicy,
) where
    H: FnMut(&BytesInput) -> ExitKind,
{
    color_backtrace::install();
    env_logger::init();
    log::info!("Starting");
    unsafe {
        // match fuzz_test(harness, &options, tauri_cmd_address, policy) {
        match fuzz(harness, &options, tauri_cmd_address, policy) {
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

    let mut run_client = |state: Option<_>, mgr: LlmpRestartingEventManager<_, _>, core_id| {
        // The restarting state will spawn the same process again as child, then restarted it each time it crashes.

        (|state: Option<_>, mut mgr: LlmpRestartingEventManager<_, _>, _core_id| {
            let gum = Gum::obtain();

             let syscall_blocker =
            SyscallIsolationRuntime::new(policy.clone(), tauri_cmd_address).unwrap();

            let coverage = CoverageRuntime::new();
            let cmplog = CmpLogRuntime::new();
            let drcov = DrCovRuntime::new();
            let mut frida_helper = FridaInstrumentationHelper::new(
                &gum,
                options,
                tuple_list!(coverage, cmplog, drcov, syscall_blocker),
            );

            // log::info!("Frida helper instantiated: {:#?}", frida_helper);

            // Create an observation channel using the coverage map
            let edges_observer = HitcountsMapObserver::new(StdMapObserver::from_mut_ptr(
                "edges",
                frida_helper.map_mut_ptr().unwrap(),
                MAP_SIZE,
            ));

            // Create an observation channel to keep track of the execution time
            let time_observer = TimeObserver::new("time");

            // Feedback to rate the interestingness of an input
            // This one is composed by two Feedbacks in OR
            let mut feedback = feedback_or!(
                // New maximization map feedback linked to the edges observer and the feedback state
                MaxMapFeedback::tracking(&edges_observer, true, false),
                // Time feedback, this one does not need a feedback state
                TimeFeedback::with_observer(&time_observer)
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
            let scheduler = IndexesLenTimeMinimizerScheduler::new(QueueScheduler::new());

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
        .broker_port(options.broker_port)
        .stdout_file(Some(&options.stdout))
        .remote_broker_addr(options.remote_broker_addr)
        // Store state after crashing, useful if we want to restart the fuzzer at a later time
        .serialize_state(false)
        .build();

    launcher.launch()
}

// Fuzz just a single iteration for testing
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

    let state = None;

    let gum = Gum::obtain();
    let coverage = CoverageRuntime::new();
    let cmplog = CmpLogRuntime::new();
    // TODO Change the way to pass the tauri app lib name
    let syscall_blocker = SyscallIsolationRuntime::new(policy.clone(), tauri_cmd_address).unwrap();

    let mut frida_helper = FridaInstrumentationHelper::new(
        &gum,
        options,
        tuple_list!(coverage, cmplog, syscall_blocker),
    );

    // log::info!("Frida helper instantiated: {:#?}", frida_helper);

    // Create an observation channel using the coverage map
    let edges_observer = HitcountsMapObserver::new(StdMapObserver::from_mut_ptr(
        "edges",
        frida_helper.map_mut_ptr().unwrap(),
        MAP_SIZE,
    ));

    // Create an observation channel to keep track of the execution time
    let time_observer = TimeObserver::new("time");

    // Feedback to rate the interestingness of an input
    // This one is composed by two Feedbacks in OR
    let mut feedback = feedback_or!(
        // New maximization map feedback linked to the edges observer and the feedback state
        MaxMapFeedback::tracking(&edges_observer, true, false),
        // Time feedback, this one does not need a feedback state
        TimeFeedback::with_observer(&time_observer)
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
    let scheduler = IndexesLenTimeMinimizerScheduler::new(QueueScheduler::new());

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
