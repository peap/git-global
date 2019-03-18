//! Configuration of git-global.
//!
//! Exports the `Config` struct, which defines the base path for finding git
//! repos on the machine, path patterns to ignore when scanning for repos, and
//! the location of a cache file to prevent scanning the filesystem every time
//! the list of known repos is needed.

use std::fs::{remove_file, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use app_dirs::{app_dir, get_app_dir, AppDataType, AppInfo};
use dirs::home_dir;
use git2;
use walkdir::{DirEntry, WalkDir};

use repo::Repo;

const APP: AppInfo = AppInfo {
    name: "git-global",
    author: "peap",
};
const CACHE_FILE: &'static str = "repos.txt";
const SETTING_BASEDIR: &'static str = "global.basedir";
const SETTING_IGNORED: &'static str = "global.ignore";

/// A container for git-global configuration options.
pub struct Config {
    /// The base directory to walk when searching for git repositories.
    pub basedir: String,

    /// Path patterns to ignore when searching for git repositories.
    pub ignored_patterns: Vec<String>,

    /// Path a cache file for git-global's usage.
    pub cache_file: PathBuf,
}

impl Config {
    /// Create a new `Config` with the default behavior, first checking global
    /// git config options in ~/.gitconfig, then using defaults:
    /// * `basedir`: `global.basedir`, or user's home directory
    /// * `ignored_patterns`: `global.ignore`, or nothing
    /// * `cache_file`: `repos.txt` in the user's XDG cache directory
    pub fn new() -> Config {
        let home_dir = home_dir()
            .expect("Could not determine home directory.")
            .to_str()
            .expect("Could not convert home directory path to str.")
            .to_string();
        let (basedir, patterns) = match git2::Config::open_default() {
            Ok(config) => (
                config.get_string(SETTING_BASEDIR).unwrap_or(home_dir),
                config
                    .get_string(SETTING_IGNORED)
                    .unwrap_or(String::new())
                    .split(",")
                    .map(|p| p.trim().to_string())
                    .collect(),
            ),
            Err(_) => (home_dir, Vec::new()),
        };
        let cache_file =
            match get_app_dir(AppDataType::UserCache, &APP, "cache") {
                Ok(mut dir) => {
                    dir.push(CACHE_FILE);
                    dir
                }
                Err(_) => panic!("TODO: work without XDG"),
            };
        Config {
            basedir: basedir,
            ignored_patterns: patterns,
            cache_file: cache_file,
        }
    }

    /// Returns all known git repos, populating the cache first, if necessary.
    pub fn get_repos(&mut self) -> Vec<Repo> {
        if !self.has_cache() {
            let repos = self.find_repos();
            self.cache_repos(&repos);
        }
        self.get_cached_repos()
    }

    /// Clears the cache of known git repos, forcing a re-scan on the next
    /// `get_repos()` call.
    pub fn clear_cache(&mut self) {
        if self.has_cache() {
            remove_file(&self.cache_file)
                .expect("Failed to delete cache file.");
        }
    }

    /// Returns `true` if this directory entry should be included in scans.
    fn filter(&self, entry: &DirEntry) -> bool {
        let entry_path = entry.path().to_str().expect("DirEntry without path.");

        self.ignored_patterns
            .iter()
            .filter(|p| p != &"")
            .fold(true, |acc, pattern| acc && !entry_path.contains(pattern))
    }

    /// Walks the configured base directory, looking for git repos.
    fn find_repos(&self) -> Vec<Repo> {
        let mut repos = Vec::new();
        let basedir = &self.basedir;
        println!(
            "Scanning for git repos under {}; this may take a while...",
            basedir
        );
        for entry in WalkDir::new(basedir)
            .into_iter()
            .filter_entry(|e| self.filter(e))
        {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_dir() && entry.file_name() == ".git"
                    {
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

    /// Returns boolean indicating if the cache file exists.
    fn has_cache(&self) -> bool {
        self.cache_file.as_path().exists()
    }

    /// Writes the given repo paths to the cache file.
    fn cache_repos(&self, repos: &Vec<Repo>) {
        if !self.cache_file.as_path().exists() {
            // Try to create the cache directory if the cache *file* doesn't
            // exist; app_dir() handles an existing directory just fine.
            match app_dir(AppDataType::UserCache, &APP, "cache") {
                Ok(_) => (),
                Err(e) => panic!("Could not create cache directory: {}", e),
            }
        }
        let mut f = File::create(&self.cache_file)
            .expect("Could not create cache file.");
        for repo in repos.iter() {
            match writeln!(f, "{}", repo.path()) {
                Ok(_) => (),
                Err(e) => panic!("Problem writing cache file: {}", e),
            }
        }
    }

    /// Returns the list of repos found in the cache file.
    fn get_cached_repos(&self) -> Vec<Repo> {
        let mut repos = Vec::new();
        if self.cache_file.as_path().exists() {
            let f = File::open(&self.cache_file)
                .expect("Could not open cache file.");
            let reader = BufReader::new(f);
            for line in reader.lines() {
                match line {
                    Ok(repo_path) => repos.push(Repo::new(repo_path)),
                    Err(_) => (), // TODO: handle errors
                }
            }
        }
        repos
    }
}
