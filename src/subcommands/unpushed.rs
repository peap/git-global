//! The `unpushed` subcommand: shows repositories that have commits not pushed to a remote

use std::sync::{mpsc, Arc};
use std::thread;

use crate::config::Config;
use crate::errors::Result;
use crate::repo::Repo;
use crate::report::Report;

/// Runs the `unpushed` subcommand.
pub fn execute(mut config: Config) -> Result<Report> {
    let repos = config.get_repos();
    let n_repos = repos.len();
    let mut report = Report::new(&repos);
    // TODO: limit number of threads, perhaps with mpsc::sync_channel(n)?
    let (tx, rx) = mpsc::channel();
    for repo in repos {
        let tx = tx.clone();
        let repo = Arc::new(repo);
        thread::spawn(move || {
            let path = repo.path();
            let synced = repo.is_origin_synced();
            tx.send((path, synced)).unwrap();
        });
    }
    for _ in 0..n_repos {
        let (path, synced) = rx.recv().unwrap();
        let repo = Repo::new(path.to_string());
        if !synced {
            report.add_repo_message(&repo, format!(""));
        }
    }
    Ok(report)
}
