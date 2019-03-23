//! Git repository representation for git-global.

use std::fmt;
use std::io::{stderr, Write};
use std::path::PathBuf;

use git2;

/// A git repository, represented by the full path to its base directory.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Repo {
    path: PathBuf,
}

impl Repo {
    pub fn new(path: String) -> Repo {
        Repo {
            path: PathBuf::from(path),
        }
    }

    /// Returns the `git2::Repository` equivalent of this repo.
    pub fn as_git2_repo(&self) -> Option<git2::Repository> {
        git2::Repository::open(&self.path).ok()
    }

    /// Returns the full path to the repo as a `String`.
    pub fn path(&self) -> String {
        self.path.to_str().unwrap().to_string()
    }

    /// Returns "short format" status output.
    pub fn get_status_lines(
        &self,
        mut status_opts: git2::StatusOptions,
    ) -> Vec<String> {
        let git2_repo = match self.as_git2_repo() {
            None => {
                writeln!(
                    &mut stderr(),
                    "Could not open {} as a git repo. Perhaps you should run \
                     `git global scan` again.",
                    self
                )
                .expect("failed to write to STDERR");
                return vec![];
            }
            Some(repo) => repo,
        };
        let statuses = git2_repo
            .statuses(Some(&mut status_opts))
            .expect(&format!("Could not get statuses for {}.", self));
        statuses
            .iter()
            .map(|entry| {
                let path = entry.path().unwrap();
                let status = entry.status();
                let status_for_path = get_short_format_status(status);
                format!("{} {}", status_for_path, path)
            })
            .collect()
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path())
    }
}

/// Translates a file's status flags to their "short format" representation.
///
/// Follows an example in the git2-rs crate's `examples/status.rs`.
fn get_short_format_status(status: git2::Status) -> String {
    let mut istatus = match status {
        s if s.is_index_new() => 'A',
        s if s.is_index_modified() => 'M',
        s if s.is_index_deleted() => 'D',
        s if s.is_index_renamed() => 'R',
        s if s.is_index_typechange() => 'T',
        _ => ' ',
    };
    let mut wstatus = match status {
        s if s.is_wt_new() => {
            if istatus == ' ' {
                istatus = '?';
            }
            '?'
        }
        s if s.is_wt_modified() => 'M',
        s if s.is_wt_deleted() => 'D',
        s if s.is_wt_renamed() => 'R',
        s if s.is_wt_typechange() => 'T',
        _ => ' ',
    };
    if status.is_ignored() {
        istatus = '!';
        wstatus = '!';
    }
    if status.is_conflicted() {
        istatus = 'C';
        wstatus = 'C';
    }
    // TODO: handle submodule statuses?
    format!("{}{}", istatus, wstatus)
}
