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
