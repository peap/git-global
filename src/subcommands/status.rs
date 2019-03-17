//! The `status` subcommand: shows `git status -s` for all known repos.

use std::io::{stderr, Write};
use std::sync::{mpsc, Arc};
use std::thread;

use git2;

use config::GitGlobalConfig;
use core::{get_repos, GitGlobalResult};
use errors::Result;
use repo::Repo;

/// Translates a file's status flags to their "short format" representation.
///
/// Follows an example in the git2-rs crate's `examples/status.rs`.
fn get_short_format_status(path: &str, status: git2::Status) -> String {
    let mut istatus = match status {
        s if s.contains(git2::STATUS_INDEX_NEW) => 'A',
        s if s.contains(git2::STATUS_INDEX_MODIFIED) => 'M',
        s if s.contains(git2::STATUS_INDEX_DELETED) => 'D',
        s if s.contains(git2::STATUS_INDEX_RENAMED) => 'R',
        s if s.contains(git2::STATUS_INDEX_TYPECHANGE) => 'T',
        _ => ' ',
    };
    let mut wstatus = match status {
        s if s.contains(git2::STATUS_WT_NEW) => {
            if istatus == ' ' {
                istatus = '?';
            }
            '?'
        }
        s if s.contains(git2::STATUS_WT_MODIFIED) => 'M',
        s if s.contains(git2::STATUS_WT_DELETED) => 'D',
        s if s.contains(git2::STATUS_WT_RENAMED) => 'R',
        s if s.contains(git2::STATUS_WT_TYPECHANGE) => 'T',
        _ => ' ',
    };
    if status.contains(git2::STATUS_IGNORED) {
        istatus = '!';
        wstatus = '!';
    }
    // TODO: handle submodule statuses?
    format!("{}{} {}", istatus, wstatus, path)
}

/// Returns "short format" output for the given repo.
fn get_status_lines(repo: Arc<Repo>) -> Vec<String> {
    let git2_repo = match repo.as_git2_repo() {
        None => {
            writeln!(
                &mut stderr(),
                "Could not open {} as a git repo. Perhaps you should run \
                 `git global scan` again.",
                repo
            )
            .expect("failed to write to STDERR");
            return vec![];
        }
        Some(repo) => repo,
    };
    let mut opts = git2::StatusOptions::new();
    opts.show(git2::StatusShow::IndexAndWorkdir)
        .include_ignored(false);
    let statuses = git2_repo
        .statuses(Some(&mut opts))
        .expect(&format!("Could not get statuses for {}.", repo));
    statuses
        .iter()
        .map(|entry| {
            let path = entry.path().unwrap();
            let status = entry.status();
            let status_for_path = get_short_format_status(path, status);
            // result.add_repo_message(repo, format!("{}", status_for_path));
            format!("{}", status_for_path)
        })
        .collect()
}

/// Gathers `git status -s` for all known repos.
pub fn get_results(mut config: GitGlobalConfig) -> Result<GitGlobalResult> {
    let repos = get_repos(&mut config);
    let n_repos = repos.len();
    let mut result = GitGlobalResult::new(&repos);
    result.pad_repo_output();
    // TOOD: limit number of threads, perhaps with mpsc::sync_channel(n)?
    let (tx, rx) = mpsc::channel();
    for repo in repos {
        let tx = tx.clone();
        let repo = Arc::new(repo);
        thread::spawn(move || {
            let path = repo.path();
            let lines = get_status_lines(repo);
            tx.send((path, lines)).unwrap();
        });
    }
    for _ in 0..n_repos {
        let (path, lines) = rx.recv().unwrap();
        let repo = Repo::new(path.to_string());
        for line in lines {
            result.add_repo_message(&repo, line);
        }
    }
    Ok(result)
}
