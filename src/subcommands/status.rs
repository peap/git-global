//! The `status` subcommand: shows `git status -s` for all known repos.

use std::io::{stderr, Write};
use std::sync::{mpsc, Arc};
use std::thread;

use git2;

use config::Config;
use errors::Result;
use repo::Repo;
use report::Report;

/// Translates a file's status flags to their "short format" representation.
///
/// Follows an example in the git2-rs crate's `examples/status.rs`.
fn get_short_format_status(path: &str, status: git2::Status) -> String {
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
    format!("{}{} {}", istatus, wstatus, path)
}

/// Returns "short format" output for the given repo.
fn get_status_lines(repo: Arc<Repo>, include_untracked: bool) -> Vec<String> {
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
        .include_untracked(include_untracked)
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
            format!("{}", status_for_path)
        })
        .collect()
}

/// Gathers `git status -s` for all known repos.
pub fn execute(mut config: Config) -> Result<Report> {
    let include_untracked = config.show_untracked;
    let repos = config.get_repos();
    let n_repos = repos.len();
    let mut report = Report::new(&repos);
    report.pad_repo_output();
    // TODO: limit number of threads, perhaps with mpsc::sync_channel(n)?
    let (tx, rx) = mpsc::channel();
    for repo in repos {
        let tx = tx.clone();
        let repo = Arc::new(repo);
        thread::spawn(move || {
            let path = repo.path();
            let lines = get_status_lines(repo, include_untracked);
            tx.send((path, lines)).unwrap();
        });
    }
    for _ in 0..n_repos {
        let (path, lines) = rx.recv().unwrap();
        let repo = Repo::new(path.to_string());
        for line in lines {
            report.add_repo_message(&repo, line);
        }
    }
    Ok(report)
}
