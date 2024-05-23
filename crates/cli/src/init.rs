use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::Parser;
use handlebars::Handlebars;
use include_dir::{include_dir, Dir};
use serde::Serialize;

use crate::util::tauri_dir;

const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/template");

fn render_with_generator<
    P: AsRef<Path>,
    S: Serialize,
    F: FnMut(PathBuf) -> std::io::Result<Option<fs::File>>,
>(
    handlebars: &mut Handlebars,
    data: &S,
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
            if let Some(contents) = file.contents_utf8() {
                let rendered = handlebars.render_template(contents, data)?;
                output_file.write_all(rendered.as_bytes())?
            } else {
                output_file.write_all(file.contents())?
            }
        }
    }

    for dir in dir.dirs() {
        render_with_generator(handlebars, data, dir, out_dir, out_file_generator)?;
    }

    Ok(())
}

fn render<P: AsRef<Path>, S: Serialize>(
    handlebars: &mut Handlebars,
    data: &S,
    dir: &Dir<'_>,
    out_dir: P,
) -> anyhow::Result<()> {
    let out_dir = out_dir.as_ref();
    let mut created_dirs = Vec::new();
    render_with_generator(handlebars, data, dir, out_dir, &mut |file_path: PathBuf| {
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
    /// Set the crate name to be fuzzed
    #[clap(short, long)]
    crate_name: Option<String>,
}

pub fn command(options: Options) -> anyhow::Result<()> {
    let cwd = match options.directory {
        Some(dir) => dir,
        None => tauri_dir()?,
    };

    let fuzz_dir = cwd.join("fuzz");
    fs::create_dir_all(&fuzz_dir)?;

    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);

    let mut data = BTreeMap::new();

    let crate_name = match options.crate_name {
        Some(name) => name,
        None => {
            let cargo_toml = std::fs::read_to_string(cwd.join("Cargo.toml"))
                .with_context(|| format!("Couldn't find `Cargo.toml` in: {}", cwd.display()))?;

            let cargo_toml: toml::Value = toml::from_str(&cargo_toml)?;
            cargo_toml
                .get("package")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Cargo.toml doesn't have a `name` field, try using `--crate-name` flag"
                    )
                })?
                .to_string()
        }
    };

    data.insert("crate_name_underscored", crate_name.replace('-', "_"));
    data.insert("crate_name", crate_name);

    render(&mut handlebars, &data, &TEMPLATE_DIR, fuzz_dir)
}
