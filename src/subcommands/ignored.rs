//! The `ignored` subcommand: lists all ignored repos.

use crate::config::Config;
use crate::errors::Result;
use crate::report::Report;

/// Lists all repos that are currently ignored.
pub fn execute(config: Config) -> Result<Report> {
    let ignored = config.get_ignored_repos();
    let mut report = Report::new(&[]);
    if ignored.is_empty() {
        report.add_message("No repos are currently ignored.".to_string());
    } else {
        report.add_message(format!("Ignored repos ({}):", ignored.len()));
        for path in ignored {
            report.add_message(format!("  {}", path));
        }
    }
    Ok(report)
}
