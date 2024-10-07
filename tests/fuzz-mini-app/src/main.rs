#![allow(unused_variables, unused_imports)]
mod utils;
use mini_app::basic::direct_panic;
use mini_app::file_access::read_foo_file;
use utils::*;

pub fn main() {
    color_backtrace::install();
    env_logger::init();
    // fuzz_command_with_arg::<()>(
    //     "direct_panic",
    //     Some(direct_panic as usize),
    //     // policies::filesystem::no_file_access(),
    //     // policies::filesystem::write_only_access(),
    //     policies::no_policy(),
    //     // vec![("path", path_to_foo())],
    //     vec![],
    //     // Some("fs".into()),
    //     None
    // );
    fuzz_command_with_arg::<()>(
        "read_foo_file",
        Some(read_foo_file as usize),
        policies::filesystem::no_file_access(),
        // policies::filesystem::write_only_access(),
        // policies::no_policy(),
        // vec![("path", path_to_foo())],
        vec![],
        // Some("fs".into()),
        None,
    );
}
