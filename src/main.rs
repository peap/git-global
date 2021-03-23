//! Entry point for the binary.

use std::process::exit;

/// Runs git-global from the command line, exiting with its return value.
fn main() {
    exit(git_global::run_from_command_line())
}
