// Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use libafl_bolts::bolts_prelude::Cores;
use libafl_bolts::cli::FuzzerOptions;
use serde::Deserialize;
use std::path::PathBuf;

/// A simplified configuration for the fuzzer
#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct SimpleFuzzerConfig {
    #[serde(skip)]
    solutions_dir: PathBuf,
    /// Where to print the fuzzer outputs
    stdout: String,
    /// Number of cores used to fuzz
    nb_of_cores: u16,
    /// Directories containing starting input to start fuzzing
    corpus_input: Vec<String>,
    /// Enable code coverage optimization
    with_coverage: bool,
    /// Port used by the fuzzer broker
    broker_port: u16,
}

/// A simplified configuration to convert into LibAFL fuzzer configuration
impl SimpleFuzzerConfig {
    /// Create a config from a toml_file
    ///
    /// Example toml_file:
    ///
    /// stdout = '/dev/stdout'
    /// nb_of_cores = 1
    /// corpus_input = []
    /// with_coverage = true
    /// broker_port = 8888
    ///
    pub fn from_toml(toml_file: PathBuf, command_name: &str, fuzz_dir: PathBuf) -> Self {
        let mut solutions_dir = fuzz_dir.clone();
        solutions_dir.push("fuzz_solutions");
        solutions_dir.push(command_name);

        let data = std::fs::read_to_string(toml_file.to_string_lossy().as_ref())
            .unwrap_or_else(|e| panic!("{:#?}: {:#?}", toml_file, e));
        let mut config: SimpleFuzzerConfig = toml::from_str(&data)
            .unwrap_or_else(|_| panic!("Failed to deserialize {:#?}", toml_file));
        config.solutions_dir = solutions_dir;
        config
    }
}

impl From<SimpleFuzzerConfig> for FuzzerOptions {
    fn from(simple: SimpleFuzzerConfig) -> Self {
        FuzzerOptions {
            input: simple.corpus_input.into_iter().map(PathBuf::from).collect(),
            stdout: simple.stdout,
            output: simple.solutions_dir,

            cores: Cores::from_cmdline(&format!("1-{}", simple.nb_of_cores)).unwrap(),
            broker_port: simple.broker_port,
            remote_broker_addr: None,

            // Settings for Frida stalker
            harness: std::env::current_exe().ok(),
            // harness: None,
            libs_to_instrument: vec![
                // std::env::current_exe().unwrap().display().to_string(),
            ],
            disable_excludes: true,
            dont_instrument: vec![],

            // Settings for code coverage
            // You have to enable the stalker to use them
            disable_coverage: !simple.with_coverage,
            cmplog: simple.with_coverage,
            cmplog_cores: Cores::from_cmdline("1").unwrap(),
            drcov: simple.with_coverage,

            // Settings for the memory sanitizer
            // We don't really use it in our fuzzer
            asan: false,
            asan_cores: Cores::from_cmdline("1").unwrap(),
            detect_leaks: false,
            continue_on_error: false,
            allocation_backtraces: true,
            max_allocation: 1073741824,
            max_total_allocation: 4294967296,
            max_allocation_panics: true,

            // Not used in LibAFL frida
            iterations: 0,
            configuration: String::from("default configuration"),
            verbose: true,
            timeout: std::time::Duration::from_secs(0),
            harness_function: String::new(),
            harness_args: vec![],
            tokens: vec![], // Certainly for input mutation
            replay: None,
            repeat: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_toml_configuration() {
        let toml_config = ["configuration", "toml_template.toml"].iter().collect();
        let config = SimpleFuzzerConfig::from_toml(toml_config, "foo", PathBuf::new());
        assert_eq!(
            config,
            SimpleFuzzerConfig {
                solutions_dir: PathBuf::from("fuzz_solutions/foo"),
                stdout: "/dev/stdout".to_string(),
                nb_of_cores: 1,
                corpus_input: vec![],
                with_coverage: true,
                broker_port: 8888
            }
        );
    }
}
