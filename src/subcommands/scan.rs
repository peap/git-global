//! The `scan` subcommand: scans the filesystem for git repos.
//!
//! By default, the user's home directory is walked, but this starting point can
//! be configured in `~/.gitconfig`:
//!
//! ```bash
//! $ git config --global global.basedir /some/path
//! ```
//!
//! Additional directories can be scanned by passing them as arguments:
//!
//! ```bash
//! $ git global scan /extra/path1 /extra/path2
//! ```
//!
//! The `scan` subcommand caches the list of git repos paths it finds, and can
//! be rerun at any time to refresh the list.

use std::path::PathBuf;

use crate::config::Config;
use crate::errors::Result;
use crate::report::Report;

/// Clears the cache, forces a rescan, and says how many repos were found.
pub fn execute(
    mut config: Config,
    extra_paths: Vec<PathBuf>,
) -> Result<Report> {
    let repos = config.scan_with_extra_paths(&extra_paths);
    let mut report = Report::new(&repos);
    report.add_message(format!(
        "Found {} repos. Use `git global list` to show them.",
        repos.len()
    ));
    Ok(report)
}
