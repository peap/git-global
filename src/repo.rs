//! Git repository representation for git-global.

use std::fmt;
use std::path::PathBuf;

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
    pub fn as_git2_repo(&self) -> ::git2::Repository {
        ::git2::Repository::open(&self.path).expect(
            "Could not open {} as a git repo. Perhaps you should run \
             `git global scan` again.",
        )
    }

    /// Returns the full path to the repo as a `String`.
    pub fn path(&self) -> String {
        self.path.to_str().unwrap().to_string()
    }

    /// Returns "short format" status output.
    pub fn get_status_lines(
        &self,
        mut status_opts: ::git2::StatusOptions,
    ) -> Vec<String> {
        let git2_repo = self.as_git2_repo();
        let statuses = git2_repo
            .statuses(Some(&mut status_opts))
            .unwrap_or_else(|_| panic!("Could not get statuses for {}.", self));
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

    /// Transforms a git2::Branch into a git2::Comit
    fn branch_to_commit(branch: git2::Branch) -> git2::Commit {
        branch.into_reference().peel_to_commit().unwrap()
    }

    /// Walks trough revisions : returns all the ancestores Oids of a Commit
    fn log(repo: &git2::Repository, commit: git2::Commit) -> Vec<git2::Oid> {
        let mut revwalk = repo.revwalk().unwrap();
        revwalk.push(commit.id()).unwrap();
        revwalk.filter_map(|id| id.ok()).collect::<Vec<git2::Oid>>()
    }

    /// Returns true if origin is synced with all the local branches, and false if not
    pub fn is_origin_synced(&self) -> bool {
        let repo = self.as_git2_repo();
        let local_branches =
            repo.branches(Some(git2::BranchType::Local)).unwrap();
        let remote_branches =
            repo.branches(Some(git2::BranchType::Remote)).unwrap();

        let remote_commit_ids = remote_branches
            .map(|result| result.unwrap().0)
            .map(Self::branch_to_commit)
            .map(|commit| Self::log(&repo, commit))
            .flatten()
            .collect::<Vec<_>>();

        let is_synced =
            local_branches
                .map(|result| result.unwrap().0)
                .all(|branch| {
                    let commit_id = Self::branch_to_commit(branch).id();
                    remote_commit_ids.contains(&commit_id)
                });
        is_synced
    }

    /// Returns the list of stash entries for the repo.
    pub fn get_stash_list(&self) -> Vec<String> {
        let mut stash = vec![];
        self.as_git2_repo()
            .stash_foreach(|index, name, _oid| {
                stash.push(format!("stash@{{{}}}: {}", index, name));
                true
            })
            .unwrap();
        stash
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
fn get_short_format_status(status: ::git2::Status) -> String {
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
