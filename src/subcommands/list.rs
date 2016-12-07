//! The `list` subcommand, which lists all repos known to git-global.

use super::super::{GitGlobalResult, Repo, Result};

pub fn get_results(repos: Vec<Repo>) -> Result<GitGlobalResult> {
    Ok(GitGlobalResult::new(&repos))
}
