use std::path::PathBuf;
const CONFIG_FILE: &str = "fuzzer_config.toml";

pub(crate) fn fuzz_config() -> PathBuf {
    let mut config_file = fuzz_dir();
    config_file.push(CONFIG_FILE);
    config_file
}

pub(crate) fn fuzz_dir() -> PathBuf {
    std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
}

#[allow(dead_code)]
pub(crate) fn path_to_foo() -> PathBuf {
    let mut assets_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    assets_dir.pop();
    assets_dir.push("assets");
    assets_dir.push("foo.txt");
    assets_dir
}
