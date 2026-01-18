//! The `ahead` subcommand: shows repositories that have commits not pushed to a remote

use crate::config::Config;
use crate::errors::Result;
use crate::parallel::{default_parallelism, run_parallel};
use crate::repo::Repo;
use crate::report::Report;

/// Runs the `ahead` subcommand.
pub fn execute(mut config: Config) -> Result<Report> {
    let repos = config.get_repos();
    let mut report = Report::new(&repos);

    let results = run_parallel(repos, default_parallelism(), |repo| repo.is_ahead());

    for (path, ahead) in results {
        if ahead {
            let repo = Repo::new(path);
            report.add_repo_message(&repo, String::new());
        }
    }

    Ok(report)
}
