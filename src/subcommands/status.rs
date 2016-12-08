//! The `status` subcommand, which shows `git status` for all known repos.

use core::{GitGlobalResult, get_repos};
use errors::Result;

pub fn get_results() -> Result<GitGlobalResult> {
    let repos = get_repos();
    Ok(GitGlobalResult::new(&repos))
}
