//! The `ignore` subcommand: adds a repo to the ignored list.

use crate::config::Config;
use crate::errors::{GitGlobalError, Result};
use crate::report::Report;

/// Adds the given repo path to the ignored repos list.
pub fn execute(config: Config, path: &str) -> Result<Report> {
    let mut report = Report::new(&[]);
    match config.ignore_repo(path) {
        Ok(()) => {
            report.add_message(format!("Ignoring repo: {}", path));
        }
        Err(e) => {
            return Err(GitGlobalError::BadSubcommand(e));
        }
    }
    Ok(report)
}
