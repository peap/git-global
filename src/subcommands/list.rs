//! The `list` subcommand, which lists all repos known to git-global.

use core::{GitGlobalResult, get_repos};
use errors::Result;

pub fn get_results() -> Result<GitGlobalResult> {
    let repos = get_repos();
    let mut result = GitGlobalResult::new(&repos);
    for repo in repos.iter() {
        // GitGlobalResult.print() already prints out the repo name if it has
        // any messages, so just add an empty string to force display of the
        // repo name.
        result.add_repo_message(repo, format!(""));
    }
    Ok(result)
}
