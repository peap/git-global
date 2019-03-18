//! Core functionality of git-global.
//!
//! Includes the `get_repos()`, `cache_repos()`, and `find_repos()` functions.

use walkdir::WalkDir;

use config::GitGlobalConfig;
use repo::Repo;

/// Walks the configured base directory, looking for git repos.
pub fn find_repos(config: &GitGlobalConfig) -> Vec<Repo> {
    let mut repos = Vec::new();
    let basedir = &config.basedir;

    println!(
        "Scanning for git repos under {}; this may take a while...",
        basedir
    );
    for entry in WalkDir::new(basedir)
        .into_iter()
        .filter_entry(|e| config.filter(e))
    {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_dir() && entry.file_name() == ".git" {
                    let parent_path = entry
                        .path()
                        .parent()
                        .expect("Could not determine parent.");
                    match parent_path.to_str() {
                        Some(path) => {
                            repos.push(Repo::new(path.to_string()));
                        }
                        None => (),
                    }
                }
            }
            Err(_) => (),
        }
    }
    repos.sort_by(|a, b| a.path().cmp(&b.path()));
    repos
}

/// Caches repo list to disk, in the XDG cache directory for git-global.
pub fn cache_repos(config: &mut GitGlobalConfig, repos: &Vec<Repo>) {
    config.cache_repos(repos);
}

/// Returns all known git repos, populating the cache first, if necessary.
pub fn get_repos(config: &mut GitGlobalConfig) -> Vec<Repo> {
    if !config.has_cache() {
        let repos = find_repos(config);
        cache_repos(config, &repos);
        repos
    } else {
        config.get_cached_repos()
    }
}
