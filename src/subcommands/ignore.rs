//! The `ignore` subcommand: adds a pattern to the global.ignore gitconfig.

use crate::config::Config;
use crate::errors::{GitGlobalError, Result};
use crate::report::Report;

/// Adds the given pattern to global.ignore in gitconfig.
pub fn execute(_config: Config, pattern: &str) -> Result<Report> {
    let mut report = Report::new(&[]);
    match Config::add_ignore_pattern(pattern) {
        Ok(()) => {
            report.add_message(format!(
                "Added '{}' to global.ignore. Run `git global scan` to update the cache.",
                pattern
            ));
        }
        Err(e) => {
            return Err(GitGlobalError::BadSubcommand(e));
        }
    }
    Ok(report)
}
