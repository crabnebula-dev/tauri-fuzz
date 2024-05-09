use clap::{ArgAction, Parser, Subcommand};
use log::Level;

mod init;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    bin_name("cargo-tauri-fuzz"),
    subcommand_required(true),
    arg_required_else_help(true),
    propagate_version(true),
    no_binary_name(true)
)]
struct Cli {
    /// Enables verbose logging.
    #[clap(short, long, global = true, action = ArgAction::Count)]
    verbose: u8,
    /// Disables logging
    #[clap(short, long)]
    quite: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init(init::Options),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse_from(std::env::args_os().skip(1));

    if !cli.quite {
        if let Err(err) = setup_logger(&cli) {
            eprintln!("Failed to attach logger: {err}");
        }
    }

    let res = match cli.command {
        Commands::Init(opts) => init::command(opts),
    };

    if let Err(e) = res {
        log::error!("{}", e);
        std::process::exit(1);
    }

    Ok(())
}

fn setup_logger(cli: &Cli) -> anyhow::Result<()> {
    use std::io::Write;
    env_logger::Builder::from_default_env()
        .filter(None, verbosity_level(cli.verbose).to_level_filter())
        .format(|buf, record| {
            let style = buf.default_level_style(record.level()).bold();
            let level = prettyprint_level(record.level());
            writeln!(buf, "{style}{}{style:#} {}", level, record.args())
        })
        .try_init()
        .map_err(Into::into)
}

/// This maps the occurrence of `--verbose` flags to the correct log level
const fn verbosity_level(num: u8) -> Level {
    match num {
        0 => Level::Info,
        1 => Level::Debug,
        2.. => Level::Trace,
    }
}

/// The default string representation for `Level` is all uppercaps which doesn't mix well with the other printed actions.
const fn prettyprint_level(lvl: Level) -> &'static str {
    match lvl {
        Level::Error => "Error",
        Level::Warn => "Warn",
        Level::Info => "Info",
        Level::Debug => "Debug",
        Level::Trace => "Trace",
    }
}
