//! Subcommand implementations and dispatch function `run()`.
pub mod info;
pub mod list;
pub mod scan;
pub mod status;

use config::Config;
use errors::{GitGlobalError, Result};
use report::Report;

/// Run the subcommand matching the provided `str`, returning a `Report`.
pub fn run(command: &str, config: Config) -> Result<Report> {
    match command {
        "info" => info::execute(config),
        "list" => list::execute(config),
        "scan" => scan::execute(config),
        "status" => status::execute(config),
        cmd => Err(GitGlobalError::BadSubcommand(cmd.to_string())),
    }
}
