//! Git repository handling, via the `Repo` struct.

use std::fmt;
use std::path::PathBuf;

use git2;

#[derive(Clone, Eq, Hash, PartialEq)]
/// A git repository, represented by the full path to its base directory.
pub struct Repo {
    path: PathBuf,
}

impl Repo {
    pub fn new(path: PathBuf) -> Repo {
        Repo {
            path: path,
        }
    }

    /// Returns the `git2::Repository` equivalent of this repo.
    pub fn as_git2_repo(&self) -> Option<git2::Repository> {
        git2::Repository::open(&self.get_path()).ok()
    }
    /// Returns the full path to the repo.
    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }

    /// Returns the full path to the repo as a `String`.
    pub fn get_path_as_string(&self) -> String {
        format!("{}", self.get_path().display())
    }

}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_path_as_string())
    }
}
