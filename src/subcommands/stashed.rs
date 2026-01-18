//! The `stashed` subcommand: shows stash list for all known repos with stashes

use crate::config::Config;
use crate::errors::Result;
use crate::parallel::{default_parallelism, run_parallel};
use crate::repo::Repo;
use crate::report::Report;

/// Runs the `stashed` subcommand.
pub fn execute(mut config: Config) -> Result<Report> {
    let repos = config.get_repos();
    let mut report = Report::new(&repos);
    report.pad_repo_output();

    let results = run_parallel(repos, default_parallelism(), |repo| {
        repo.get_stash_list()
    });

    for (path, stash) in results {
        let repo = Repo::new(path);
        for line in stash {
            report.add_repo_message(&repo, line);
        }
    }

    Ok(report)
}
