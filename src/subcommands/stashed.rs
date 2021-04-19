//! The `stashed` subcommand: shows stash list for all known repos with stashes

use std::sync::{mpsc, Arc};
use std::thread;

use crate::config::Config;
use crate::errors::Result;
use crate::repo::Repo;
use crate::report::Report;

/// Runs the `stashed` subcommand.
pub fn execute(mut config: Config) -> Result<Report> {
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
            let stash = repo.get_stash_list();
            tx.send((path, stash)).unwrap();
        });
    }
    for _ in 0..n_repos {
        let (path, stash) = rx.recv().unwrap();
        let repo = Repo::new(path.to_string());
        for line in stash {
            report.add_repo_message(&repo, line);
        }
    }
    Ok(report)
}
