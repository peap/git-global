//! The `unstaged` subcommand: shows `git status -s` for unstaged changes in all
//! known repos with such changes.

use std::sync::{mpsc, Arc};
use std::thread;

use crate::config::Config;
use crate::errors::Result;
use crate::repo::Repo;
use crate::report::Report;

/// Runs the `unstaged` subcommand.
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
            let mut status_opts = ::git2::StatusOptions::new();
            status_opts
                .show(::git2::StatusShow::Workdir)
                .include_untracked(include_untracked)
                .include_ignored(false);
            let lines = repo.get_status_lines(status_opts);
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
