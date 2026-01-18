//! The `staged` subcommand: shows `git status -s` for staged changes in all
//! known repos with such changes.

use crate::config::Config;
use crate::errors::Result;
use crate::parallel::{default_parallelism, run_parallel};
use crate::repo::Repo;
use crate::report::Report;

/// Runs the `staged` subcommand.
pub fn execute(mut config: Config) -> Result<Report> {
    let include_untracked = config.show_untracked;
    let repos = config.get_repos();
    let mut report = Report::new(&repos);
    report.pad_repo_output();

    let results = run_parallel(repos, default_parallelism(), move |repo| {
        let mut status_opts = git2::StatusOptions::new();
        status_opts
            .show(git2::StatusShow::Index)
            .include_untracked(include_untracked)
            .include_ignored(false);
        repo.get_status_lines(status_opts)
    });

    for (path, lines) in results {
        let repo = Repo::new(path);
        for line in lines {
            report.add_repo_message(&repo, line);
        }
    }

    Ok(report)
}
