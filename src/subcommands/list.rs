//! The `list` subcommand, which lists all repos known to git-global.

use core::{GitGlobalResult, get_repos};
use errors::Result;

pub fn get_results() -> Result<GitGlobalResult> {
    let repos = get_repos();
    Ok(GitGlobalResult::new(&repos))
}
