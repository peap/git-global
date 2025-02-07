//! The `info` subcommand: shows metadata about the git-global installation.

use clap::crate_version;

use std::env;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::config::Config;
use crate::errors::Result;
use crate::report::Report;

/// Returns the age of a file in terms of days, hours, minutes, and seconds.
fn get_age(filename: PathBuf) -> Option<String> {
    filename
        .metadata()
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|mtime| SystemTime::now().duration_since(mtime).ok())
        .map(|dur| {
            let ts = dur.as_secs();
            let days = ts / (24 * 60 * 60);
            let hours = (ts / (60 * 60)) - (days * 24);
            let mins = (ts / 60) - (days * 24 * 60) - (hours * 60);
            let secs =
                ts - (days * 24 * 60 * 60) - (hours * 60 * 60) - (mins * 60);
            format!("{}d, {}h, {}m, {}s", days, hours, mins, secs)
        })
}

/// Gathers metadata about the git-global installation.
pub fn execute(mut config: Config) -> Result<Report> {
    let repos = config.get_repos();
    let mut report = Report::new(&repos);
    let version = crate_version!().to_string();
    // beginning of underline:   git-global x.x.x
    let mut underline = "===========".to_string();
    for _ in 0..version.len() {
        underline.push('=');
    }
    report.add_message(format!("git-global {}", version));
    report.add_message(underline);
    report.add_message(format!("Number of repos: {}", repos.len()));
    report.add_message(format!("Base directory: {}", config.basedir.display()));
    report.add_message("Ignored patterns:".to_string());
    for pat in config.ignored_patterns.iter() {
        report.add_message(format!("  {}", pat));
    }
    report.add_message(format!("Default command: {}", config.default_cmd));
    report.add_message(format!("Verbose: {}", config.verbose));
    report.add_message(format!("Show untracked: {}", config.show_untracked));
    if let Some(cache_file) = config.cache_file {
        report.add_message(format!("Cache file: {}", cache_file.display()));
        if let Some(age) = get_age(cache_file) {
            report.add_message(format!("Cache file age: {}", age));
        }
    } else {
        report.add_message("Cache file: <none>".to_string());
    }
    if let Some(manpage_file) = config.manpage_file {
        report.add_message(format!("Manpage file: {}", manpage_file.display()));
    } else {
        report.add_message("Manpage file: <none>".to_string());
    }
    report.add_message(format!("Detected OS: {}", env::consts::OS));
    Ok(report)
}
