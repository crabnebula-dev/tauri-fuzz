use std::{
    cmp::Ordering,
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::Parser;
use ignore::WalkBuilder;
use include_dir::{include_dir, Dir};

const TAURI_JSON: &str = "tauri.conf.json";
const TAURI_JSON5: &str = "tauri.conf.json5";
const TAURI_TOML: &str = "Tauri.toml";

fn folder_has_tuari_config(folder: &Path) -> bool {
    folder.join(TAURI_JSON).exists()
        || folder.join(TAURI_JSON5).exists()
        || folder.join(TAURI_TOML).exists()
}

fn is_tauri_config_file(path: &Path) -> bool {
    let file_name = path.file_name();
    file_name == Some(OsStr::new(TAURI_JSON))
        || file_name == Some(OsStr::new(TAURI_JSON5))
        || file_name == Some(OsStr::new(TAURI_TOML))
}

pub fn tauri_dir() -> anyhow::Result<PathBuf> {
    let Ok(cwd) = std::env::current_dir() else {
        anyhow::bail!("failed to get current working dir");
    };

    if cwd.join(TAURI_JSON).exists()
        || cwd.join(TAURI_JSON5).exists()
        || cwd.join(TAURI_TOML).exists()
    {
        return Ok(cwd);
    }

    let src_tauri = cwd.join("src-tauri");
    if src_tauri.join(TAURI_JSON).exists()
        || src_tauri.join(TAURI_JSON5).exists()
        || src_tauri.join(TAURI_TOML).exists()
    {
        return Ok(src_tauri);
    }

    let depth = match std::env::var("TAURI_FUZZER_CONFIG_LOOKUP_DEPTH") {
        Ok(d) =>  d.parse().map_err(|_| anyhow::anyhow!("`TAURI_FUZZER_CONFIG_LOOKUP_DEPTH` environment variable must be a positive integer"))?,
        Err(_) => 3,
    };

    let mut builder = WalkBuilder::new(&cwd);
    builder
        .require_git(false)
        .ignore(false)
        .max_depth(Some(depth))
        .sort_by_file_path(|a, _| {
            if a.extension().is_some() {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

    for entry in builder.build().flatten() {
        let path = cwd.join(entry.path());
        if path.is_dir() && folder_has_tuari_config(&path) {
            return Ok(path);
        } else if is_tauri_config_file(&path) {
            return path
                .parent()
                .map(|p| p.to_path_buf())
                .context("failed to get parent from path");
        }
    }

    anyhow::bail!("Couldn't recognize the current folder as a Tauri project. It must contain a `{TAURI_JSON}`, `{TAURI_JSON5}` or `{TAURI_TOML}` file in any subfolder.")
}

const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/template");

fn render_with_generator<P: AsRef<Path>, F: FnMut(PathBuf) -> std::io::Result<Option<fs::File>>>(
    dir: &Dir<'_>,
    out_dir: P,
    out_file_generator: &mut F,
) -> anyhow::Result<()> {
    let out_dir = out_dir.as_ref();
    for file in dir.files() {
        let mut file_path = file.path().to_path_buf();
        // cargo for some reason ignores the /template folder packaging when it has a Cargo.toml file inside
        // so we rename the extension to `.crate-manifest`
        if let Some(extension) = file_path.extension() {
            if extension == "crate-manifest" {
                file_path.set_extension("toml");
            }
        }

        if let Some(mut output_file) = out_file_generator(file_path)? {
            output_file.write_all(file.contents())?
        }
    }

    for dir in dir.dirs() {
        render_with_generator(dir, out_dir, out_file_generator)?;
    }

    Ok(())
}

fn render<P: AsRef<Path>>(dir: &Dir<'_>, out_dir: P) -> anyhow::Result<()> {
    let out_dir = out_dir.as_ref();
    let mut created_dirs = Vec::new();
    render_with_generator(dir, out_dir, &mut |file_path: PathBuf| {
        let path = out_dir.join(file_path);
        let parent = path.parent().unwrap().to_path_buf();
        if !created_dirs.contains(&parent) {
            fs::create_dir_all(&parent)?;
            created_dirs.push(parent);
        }
        fs::File::create(path).map(Some)
    })
}

#[derive(Parser, Debug)]
pub struct Options {
    /// Set target directory for init
    #[clap(short, long)]
    directory: Option<PathBuf>,
}

pub fn command(options: Options) -> anyhow::Result<()> {
    let dir = match options.directory {
        Some(dir) => dir,
        None => tauri_dir()?,
    };

    let dir = dir.join("fuzz");

    fs::create_dir_all(&dir)?;
    render(&TEMPLATE_DIR, dir)
}
