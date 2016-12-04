//! core: core functionality for the `git-global` command

use std::collections::HashMap;

use super::{Result};

/// A git repo.
pub struct Repo {
    path: String,
}

impl Repo {
    pub fn new(path: String) -> Repo {
        Repo { path: path }
    }
}

/// The result of a `git-global` subcommand.
pub struct GitGlobalResult {
    repos: Vec<Repo>,
    data: HashMap<String, Vec<String>>,
}

impl GitGlobalResult {
    pub fn new() -> GitGlobalResult {
        GitGlobalResult { repos: Vec::new(), data: HashMap::new() }
    }
}

/// Scan the machine for git repos.
pub fn get_repos() -> Vec<Repo> {
    Vec::new()
}
