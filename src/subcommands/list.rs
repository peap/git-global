//! The `list` subcommand: lists all repos known to git-global.

use config::GitGlobalConfig;
use core::get_repos;
use errors::Result;
use report::Report;

/// Forces the display of each repo path, without any extra output.
pub fn execute(mut config: GitGlobalConfig) -> Result<Report> {
    let repos = get_repos(&mut config);
    let mut result = Report::new(&repos);
    for repo in repos.iter() {
        // Report.print() already prints out the repo name if it has any
        // messages, so just add an empty string to force display of the repo
        // name.
        result.add_repo_message(repo, format!(""));
    }
    Ok(result)
}
