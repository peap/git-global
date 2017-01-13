//! The `scan` subcommand, which scans the user's home directory for git repos
//! and caches the list of paths it finds.

use core::{GitGlobalResult, cache_repos, find_repos};
use errors::Result;

pub fn get_results() -> Result<GitGlobalResult> {
    let repos = find_repos();
    cache_repos(&repos);
    let mut result = GitGlobalResult::new(&repos);
    result.add_message(format!(
        "Found {} repos. Use `git global list` to show them.",
        repos.len()
    ));
    Ok(result)
}
