//! The `list` subcommand, which lists all repos known to git-global.

use core::{GitGlobalResult, cache_repos, find_repos};
use errors::Result;

pub fn get_results() -> Result<GitGlobalResult> {
    let repos = find_repos();
    let result = GitGlobalResult::new(&repos);
    cache_repos(&repos);
    Ok(result)
}
