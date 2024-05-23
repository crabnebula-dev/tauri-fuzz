use std::path::PathBuf;

use clap::Parser;

use crate::util::tauri_dir;

#[derive(Parser, Debug)]
pub struct Options {
    /// Set target directory for init
    #[clap(short, long)]
    directory: Option<PathBuf>,

    /// The target to be fuzzed. This is usually the name of the fuzz target binary
    /// defined in Cargo.toml
    fuzz_target: String,
}

pub fn command(options: Options) -> anyhow::Result<()> {
    let cwd = match options.directory {
        Some(dir) => dir,
        None => tauri_dir()?,
    };

    let fuzz_dir = cwd.join("fuzz");
    if !fuzz_dir.exists() {
        anyhow::bail!(
            "Couldn't find `fuzz` directory in {}, did you forget to run `cargo-tauri-fuzz init`?",
            cwd.display()
        )
    }

    std::process::Command::new("cargo")
        .args(["run", "--bin"])
        .arg(options.fuzz_target)
        .current_dir(fuzz_dir)
        .status()
        .map(|_| ())
        .map_err(Into::into)
}
