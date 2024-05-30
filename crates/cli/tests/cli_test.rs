use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::{Process, System};

const FUZZ_TARGETS_DECLARATION: &str = r#"
[[bin]]
name = "greet"
path = "fuzz_targets/_template_.rs"
doc = false

[[bin]]
name = "greet_full"
path = "fuzz_targets/_template_full_.rs"
doc = false
"#;

#[cfg(target_os = "linux")]
#[test]
fn init_and_fuzz() {
    let root_dir = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let tests_dir = root_dir.join(Path::new("tests"));
    let app_dir = tests_dir.join(Path::new("tauri-app"));
    let fuzz_dir = app_dir.join(["src-tauri", "fuzz"].iter().collect::<PathBuf>());
    let binaries_dir = fuzz_dir.join(["target", "debug"].iter().collect::<PathBuf>());
    let fuzz_targets = ["greet", "greet_full"];
    let fuzz_targets = ["greet_full"];

    // Build the cli
    Command::new("cargo")
        .args(["build", "-p", "fuzzer-cli"])
        .output()
        .expect("failed to build [fuzzer-cli]");

    // Init fuzz direcoty in tauri-app
    Command::new("cargo")
        .args(["run", "-p", "fuzzer-cli", "init"])
        .current_dir(app_dir.clone())
        .output()
        .expect("failed to init fuzz directory");

    // Fuzz directory has been created
    assert!(fuzz_dir.is_dir());

    // Create fuzz target
    // let template_path = fuzz_dir.join(Path::new("_template_.rs"));
    // let template_full_path = fuzz_dir.join(Path::new("_template_full_.rs"));

    // Declare the binaries in Cargo.toml
    let cargo_path = fuzz_dir.join(Path::new("Cargo.toml"));
    let mut cargo_file = OpenOptions::new()
        .append(true)
        .open(cargo_path)
        .expect("Unable to open fuzz/Cargo.toml");
    cargo_file
        .write_all(FUZZ_TARGETS_DECLARATION.as_bytes())
        .expect("Failed to declare fuzz target in fuzz/Cargo.toml");

    // Build the fuzz targets
    let mut build_commands = vec![];
    for target in fuzz_targets.iter() {
        let build_cmd = Command::new("cargo")
            .args(["build", "-p", "tauri-app-fuzz", "--bin", target])
            .current_dir(fuzz_dir.clone())
            .status()
            .unwrap_or_else(|_| panic!("Build of [{}] was terminated by signal", target));
        build_commands.push(build_cmd);
    }

    // Check if the builds were succesful
    assert!(build_commands.iter().all(|build_cmd| build_cmd.success()));

    // Start fuzzing
    // We fuzz by calling the binary directly, this avoids having to wait for compile time from
    // `cargo run`
    let mut fuzz_commands = vec![];
    for target in fuzz_targets.iter() {
        let binary = binaries_dir.join(PathBuf::from(target));
        let fuzz_cmd = Command::new(binary.to_str().unwrap())
            .spawn()
            .expect("Failed to fuzz with full template");
        fuzz_commands.push(fuzz_cmd);
    }

    // Wait for the fuzzer to start
    // This has to be long enough so that cargo has time to compile the binaries
    let one_sec = std::time::Duration::from_secs(1);
    std::thread::sleep(one_sec);

    // Kill the fuzzing processes, the fuzzers should not have exited before
    let s = System::new_all();
    for target in fuzz_targets.iter() {
        let fuzz_processes = s.processes_by_exact_name(target).collect::<Vec<&Process>>();
        for proc in fuzz_processes.into_iter() {
            proc.kill();
        }
    }

    // Clean the fuzz directory and check that it worked
    assert!(std::fs::remove_dir_all(fuzz_dir).is_ok());
}
