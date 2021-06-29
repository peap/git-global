//! Subcommand implementations and dispatch function `run()`.
pub mod ahead;
pub mod info;
pub mod list;
pub mod scan;
pub mod staged;
pub mod stashed;
pub mod status;
pub mod unstaged;

use crate::config::Config;
use crate::errors::{GitGlobalError, Result};
use crate::report::Report;

/// Run a subcommand, returning a `Report`.
///
/// If `None` is given for the optional subcommand, run `config.default_cmd`.
/// Else, try to match the given `&str` to a known subcommand.
pub fn run(maybe_subcmd: Option<&str>, config: Config) -> Result<Report> {
    let command = maybe_subcmd.unwrap_or(&config.default_cmd);
    match command {
        "info" => info::execute(config),
        "list" => list::execute(config),
        "scan" => scan::execute(config),
        "staged" => staged::execute(config),
        "stashed" => stashed::execute(config),
        "status" => status::execute(config),
        "unstaged" => unstaged::execute(config),
        "ahead" => ahead::execute(config),
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
        (
            "ahead",
            "Shows repos with changes that are not pushed to a remote",
        ),
    ]
}
