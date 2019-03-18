//! The `scan` subcommand: scans the filesystem for git repos.
//!
//! By default, the user's home directory is walked, but this starting point can
//! be configured in `~/.gitconfig`:
//!
//! ```bash
//! $ git config --global global.basedir /some/path
//! ```
//!
//! The `scan` subcommand caches the list of git repos paths it finds, and can
//! be rerun at any time to refresh the list.

use config::GitGlobalConfig;
use errors::Result;
use report::Report;

/// Clears the cache, forces a rescan, and says how many repos were found.
pub fn execute(mut config: GitGlobalConfig) -> Result<Report> {
    config.clear_cache();
    let repos = config.get_repos();
    let mut report = Report::new(&repos);
    report.add_message(format!(
        "Found {} repos. Use `git global list` to show them.",
        repos.len()
    ));
    Ok(report)
}
