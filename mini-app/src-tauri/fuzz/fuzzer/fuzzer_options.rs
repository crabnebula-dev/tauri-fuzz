use libafl_bolts::bolts_prelude::Cores;
use libafl_bolts::cli::FuzzerOptions;
use std::path::PathBuf;
use std::str::FromStr;

pub fn get_fuzzer_options(tauri_command: &str, fuzz_dir: PathBuf) -> FuzzerOptions {
    let mut solutions_dir = fuzz_dir.clone();
    solutions_dir.push("fuzz_solutions");
    solutions_dir.push(tauri_command);

    FuzzerOptions {
        timeout: std::time::Duration::from_secs(5),
        verbose: true,
        stdout: String::from("/dev/stdout"),
        configuration: String::from("default configuration"),
        asan: false,
        asan_cores: Cores::from_cmdline("1").unwrap(),
        iterations: 0,
        harness: Some(PathBuf::from_str(tauri_command).unwrap()),
        harness_args: vec![],
        harness_function: tauri_command.into(),
        libs_to_instrument: vec![],
        cmplog: true,
        cmplog_cores: Cores::from_cmdline("1").unwrap(),
        detect_leaks: false,
        continue_on_error: false,
        allocation_backtraces: true,
        max_allocation: 1073741824,
        max_total_allocation: 4294967296,
        max_allocation_panics: true,
        disable_coverage: false,
        drcov: true,
        disable_excludes: true,
        dont_instrument: vec![],
        tokens: vec![], // check
        // input: vec![PathBuf::from_str("tauri_cmd_2_fuzz/corpus").unwrap()],
        input: vec![],
        output: solutions_dir,
        // Doesn't work on MacOS
        // cores: Cores::from_cmdline("0").unwrap(),
        cores: Cores::from_cmdline("1-4").unwrap(),
        // cores: Cores::from_cmdline("1").unwrap(),
        broker_port: 8888,
        remote_broker_addr: None,
        replay: None,
        repeat: None,
    }
}
