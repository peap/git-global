//! The `ignored` subcommand: lists all patterns in global.ignore.

use crate::config::Config;
use crate::errors::Result;
use crate::report::Report;

/// Lists all patterns currently in global.ignore.
pub fn execute(config: Config) -> Result<Report> {
    let patterns: Vec<&String> = config
        .ignored_patterns
        .iter()
        .filter(|p| !p.is_empty())
        .collect();

    let mut report = Report::new(&[]);
    if patterns.is_empty() {
        report.add_message("No patterns in global.ignore.".to_string());
        report.add_message("Use `git global ignore <pattern>` to add one.".to_string());
    } else {
        report.add_message(format!("Ignored patterns ({}):", patterns.len()));
        for pattern in patterns {
            report.add_message(format!("  {}", pattern));
        }
    }
    Ok(report)
}
