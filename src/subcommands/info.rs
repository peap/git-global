//! The `info` subcommand: shows metadata about the git-global installation.

use chrono::Duration;

use std::path::PathBuf;
use std::time::SystemTime;

use config::Config;
use errors::Result;
use report::Report;

/// Returns the age of a file in terms of days, hours, minutes, and seconds.
fn get_age(filename: PathBuf) -> Option<String> {
    filename
        .metadata()
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|mtime| SystemTime::now().duration_since(mtime).ok())
        .and_then(|dur| Duration::from_std(dur).ok())
        .and_then(|dur| {
            let days = dur.num_days();
            let hours = dur.num_hours() - (days * 24);
            let mins = dur.num_minutes() - (days * 24 * 60) - (hours * 60);
            let secs = dur.num_seconds()
                - (days * 24 * 60 * 60)
                - (hours * 60 * 60)
                - (mins * 60);
            Some(format!("{}d, {}h, {}m, {}s", days, hours, mins, secs))
        })
}

/// Gathers metadata about the git-global installation.
pub fn execute(mut config: Config) -> Result<Report> {
    let repos = config.get_repos();
    let mut report = Report::new(&repos);
    let version = format!("{}", crate_version!());
    // beginning of underline:   git-global x.x.x
    let mut underline = format!("===========");
    for _ in 0..version.len() {
        underline.push('=');
    }
    report.add_message(format!("git-global {}", version));
    report.add_message(underline);
    report.add_message(format!("Number of repos: {}", repos.len()));
    report.add_message(format!("Base directory: {}", config.basedir.display()));
    report.add_message(format!("Cache file: {}", config.cache_file.display()));
    if let Some(age) = get_age(config.cache_file) {
        report.add_message(format!("Cache file age: {}", age));
    }
    report.add_message(format!("Ignored patterns:"));
    for pat in config.ignored_patterns.iter() {
        report.add_message(format!("  {}", pat));
    }
    report.add_message(format!("Default command: {}", config.default_cmd));
    report.add_message(format!("Show untracked: {}", config.show_untracked));
    Ok(report)
}
