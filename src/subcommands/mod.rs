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

/// Return the list of all subcommand names and descriptions.
///
/// Used for building the clap::App in the cli module.
pub fn get_subcommands() -> Vec<(&'static str, &'static str)> {
    vec![
        ("info", "Shows meta-information about git-global"),
        ("list", "Lists all known git repos"),
        ("scan", "Updates cache of known git repos"),
        ("status", "Shows status of all known git repos"),
    ]
}
