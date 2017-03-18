//! The `list` subcommand: lists all repos known to git-global.

use config::Config;
use errors::Result;
use subcommands::SubcommandReport;

/// Forces the display of each repo path, without any extra output.
pub fn run(config: &Config) -> Result<SubcommandReport> {
    let repos = config.get_repos();
    let mut result = SubcommandReport::new(&repos);
    for repo in repos.iter() {
        // SubcommandReport.print() already prints out the repo name if it has
        // any messages, so just add an empty string to force display of the
        // repo name.
        result.add_repo_message(repo, format!(""));
    }
    Ok(result)
}
