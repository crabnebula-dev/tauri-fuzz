// // Copyright 2023-2024 CrabNebula Ltd., Alexandre Dang
// // SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use std::{
    cmp::Ordering,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::Context;
use ignore::WalkBuilder;

pub const TAURI_JSON: &str = "tauri.conf.json";
pub const TAURI_JSON5: &str = "tauri.conf.json5";
pub const TAURI_TOML: &str = "Tauri.toml";

pub fn folder_has_tauri_config(folder: &Path) -> bool {
    folder.join(TAURI_JSON).exists()
        || folder.join(TAURI_JSON5).exists()
        || folder.join(TAURI_TOML).exists()
}

pub fn is_tauri_config_file(path: &Path) -> bool {
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
        if path.is_dir() && folder_has_tauri_config(&path) {
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
