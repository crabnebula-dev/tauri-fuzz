use std::path::PathBuf;

use clap::Parser;

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
        None => {
            let fuzz_dir = std::env::current_dir()?.join("fuzz");
            if !fuzz_dir.exists() {
                anyhow::bail!("Couldn't find `fuzz` directory in the current directory")
            }
            fuzz_dir
        }
    };

    std::process::Command::new("cargo")
        .args(["run", "--bin"])
        .arg(options.fuzz_target)
        .current_dir(cwd)
        .status()
        .map(|_| ())
        .map_err(Into::into)
}
