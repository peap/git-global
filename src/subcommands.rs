//! Subcommand implementations and dispatch function `run()`.
pub mod ahead;
pub mod ignore;
pub mod ignored;
pub mod info;
pub mod install_manpage;
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
/// The `args` parameter is used for subcommands that require additional arguments.
pub fn run(
    maybe_subcmd: Option<&str>,
    config: Config,
    args: Option<&str>,
) -> Result<Report> {
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
        "install-manpage" => install_manpage::execute(config),
        "ignore" => {
            let path = args.ok_or_else(|| {
                GitGlobalError::BadSubcommand("ignore requires a path argument".to_string())
            })?;
            ignore::execute(config, path)
        }
        "ignored" => ignored::execute(config),
        cmd => Err(GitGlobalError::BadSubcommand(cmd.to_string())),
    }
}

/// Return the list of all subcommand names and descriptions.
///
/// Used for building the clap::Command in the cli module.
pub fn get_subcommands() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "ahead",
            "Shows repos with changes that are not pushed to a remote",
        ),
        ("ignore", "Ignores a repo, removing it from the list"),
        ("ignored", "Lists all ignored repos"),
        ("info", "Shows meta-information about git-global"),
        (
            "install-manpage",
            "Attempts to install git-global's man page",
        ),
        ("list", "Lists all known repos"),
        ("scan", "Updates cache of known repos"),
        (
            "staged",
            "Shows git index status for repos with staged changes",
        ),
        ("stashed", "Shows repos with stashed changes"),
        (
            "status",
            "Shows status (`git status -s`) for repos with any changes",
        ),
        (
            "unstaged",
            "Shows working dir status for repos with unstaged changes",
        ),
    ]
}
