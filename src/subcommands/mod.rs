//! Subcommand implementations and dispatch function `run()`.
pub mod info;
pub mod list;
pub mod scan;
pub mod staged;
pub mod stashed;
pub mod status;
pub mod unstaged;

use config::Config;
use errors::{GitGlobalError, Result};
use report::Report;

/// Run the subcommand matching the provided `str`, returning a `Report`.
pub fn run(command: &str, config: Config) -> Result<Report> {
    match command {
        "info" => info::execute(config),
        "list" => list::execute(config),
        "scan" => scan::execute(config),
        "staged" => staged::execute(config),
        "stashed" => stashed::execute(config),
        "status" => status::execute(config),
        "unstaged" => unstaged::execute(config),
        cmd => Err(GitGlobalError::BadSubcommand(cmd.to_string())),
    }
}

/// Return the list of all subcommand names and descriptions.
///
/// Used for building the clap::App in the cli module.
pub fn get_subcommands() -> Vec<(&'static str, &'static str)> {
    vec![
        ("info", "Shows meta-information about git-global"),
        ("list", "Lists all known repos"),
        ("scan", "Updates cache of known repos"),
        (
            "staged",
            "Show git index status for repos with staged changes",
        ),
        ("stashed", "Shows repos with stashed changes"),
        (
            "status",
            "Shows status (`git status -s`) for repos with any changes",
        ),
        (
            "unstaged",
            "Show working dir status for repos with unstaged changes",
        ),
    ]
}
