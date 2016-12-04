extern crate git_global;

use std::process::exit;

fn main() {
    exit(git_global::run_from_command_line())
}
