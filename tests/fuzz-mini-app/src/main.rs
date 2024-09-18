#![allow(unused_variables)]
use fuzz_mini_app::utils::fuzz_command_with_arg;
mod utils;
use utils::*;

pub fn main() {
    color_backtrace::install();
    env_logger::init();
    log::info!("toto");
    fuzz_command_with_arg(
        "read_file",
        None,
        // policies::filesystem::no_file_access(),
        policies::filesystem::write_only_access(),
        // policies::no_policy(),
        vec![("path", path_to_foo())],
        Some("fs".into()),
    );
}
