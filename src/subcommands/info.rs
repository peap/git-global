//! The `info` subcommand, which shows meta-information about git-global.

use core::{GitGlobalConfig, GitGlobalResult, get_repos};
use errors::Result;

pub fn get_results() -> Result<GitGlobalResult> {
    let repos = get_repos();
    let mut result = GitGlobalResult::new(&repos);
    let config = GitGlobalConfig::new();
    result.add_message(format!("git-global {}", crate_version!()));
    result.add_message(format!("=================="));
    result.add_message(format!("Number of repos: {}", repos.len()));
    result.add_message(format!("Base directory: {}", config.basedir));
    result.add_message(format!("Cache file: {}", config.cache_file.to_str().unwrap()));
    result.add_message(format!("Ignored patterns:"));
    for pat in config.ignored_patterns.iter() {
        result.add_message(format!("  {}", pat));
    }
    Ok(result)
}
