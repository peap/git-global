//! Git repository representation for git-global.

use std::fmt;

use git2;

/// A git repository, represented by the full path to its base directory.
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Repo {
    path: String,
}

impl Repo {
    pub fn new(path: String) -> Repo {
        Repo {
            path: path,
        }
    }

    /// Returns the full path to the repo as a `String`.
    pub fn path(&self) -> String {
        self.path.clone()
    }

    /// Returns the `git2::Repository` equivalent of this repo.
    pub fn as_git2_repo(&self) -> Option<git2::Repository> {
        git2::Repository::open(&self.path).ok()
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}
