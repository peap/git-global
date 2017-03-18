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

use config::Config;
use errors::Result;
use subcommands::Report;

/// Caches the results of `find_repos()` and says how many were found.
pub fn run(config: &Config) -> Result<Report> {
    config.update_cache();
    let repos = config.get_repos();
    let mut result = Report::new(&repos);
    result.add_message(format!(
        "Found {} repos. Use `git global list` to show them.",
        repos.len()
    ));
    Ok(result)
}
