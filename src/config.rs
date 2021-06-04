//! Configuration of git-global.
//!
//! Exports the `Config` struct, which defines the base path for finding git
//! repos on the machine, path patterns to ignore when scanning for repos, the
//! location of a cache file, and other config options for running git-global.

use std::fs::{create_dir_all, remove_file, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use directories::{ProjectDirs, UserDirs};
use walkdir::{DirEntry, WalkDir};

use crate::repo::Repo;

const QUALIFIER: &str = "";
const ORGANIZATION: &str = "peap";
const APPLICATION: &str = "git-global";
const CACHE_FILE: &str = "repos.txt";

const DEFAULT_CMD: &str = "status";
const DEFAULT_FOLLOW_SYMLINKS: bool = true;
const DEFAULT_SAME_FILESYSTEM: bool = cfg!(any(unix, windows));
const DEFAULT_SHOW_UNTRACKED: bool = true;

const SETTING_BASEDIR: &str = "global.basedir";
const SETTING_FOLLOW_SYMLINKS: &str = "global.follow-symlinks";
const SETTING_SAME_FILESYSTEM: &str = "global.same-filesystem";
const SETTING_IGNORE: &str = "global.ignore";
const SETTING_DEFAULT_CMD: &str = "global.default-cmd";
const SETTING_SHOW_UNTRACKED: &str = "global.show-untracked";

/// A container for git-global configuration options.
pub struct Config {
    /// The base directory to walk when searching for git repositories.
    ///
    /// Default: $HOME.
    pub basedir: PathBuf,

    /// Whether to follow symbolic links when searching for git repos.
    ///
    /// Default: true
    pub follow_symlinks: bool,

    /// Whether to stay on the same filesystem (as `basedir`) when searching
    /// for git repos on Unix or Windows.
    ///
    /// Default: true [on supported platforms]
    pub same_filesystem: bool,

    /// Path patterns to ignore when searching for git repositories.
    ///
    /// Default: none
    pub ignored_patterns: Vec<String>,

    /// The git-global subcommand to run when unspecified.
    ///
    /// Default: `status`
    pub default_cmd: String,

    /// Whether to show untracked files in output.
    ///
    /// Default: true
    pub show_untracked: bool,

    /// Optional path to a cache file for git-global's usage.
    ///
    /// Default: `repos.txt` in the user's XDG cache directory, if we understand
    /// XDG for the host system.
    pub cache_file: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}

impl Config {
    /// Create a new `Config` with the default behavior, first checking global
    /// git config options in ~/.gitconfig, then using defaults:
    pub fn new() -> Self {
        // Find the user's home directory.
        let homedir = UserDirs::new()
            .expect("Could not determine home directory.")
            .home_dir()
            .to_path_buf();
        // Set the options that aren't user-configurable.
        let cache_file =
            ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
                .map(|project_dirs| project_dirs.cache_dir().join(CACHE_FILE));
        match ::git2::Config::open_default() {
            Ok(cfg) => Config {
                basedir: cfg.get_path(SETTING_BASEDIR).unwrap_or(homedir),
                follow_symlinks: cfg
                    .get_bool(SETTING_FOLLOW_SYMLINKS)
                    .unwrap_or(DEFAULT_FOLLOW_SYMLINKS),
                same_filesystem: cfg
                    .get_bool(SETTING_SAME_FILESYSTEM)
                    .unwrap_or(DEFAULT_SAME_FILESYSTEM),
                ignored_patterns: cfg
                    .get_string(SETTING_IGNORE)
                    .unwrap_or_default()
                    .split(',')
                    .map(|p| p.trim().to_string())
                    .collect(),
                default_cmd: cfg
                    .get_string(SETTING_DEFAULT_CMD)
                    .unwrap_or_else(|_| String::from(DEFAULT_CMD)),
                show_untracked: cfg
                    .get_bool(SETTING_SHOW_UNTRACKED)
                    .unwrap_or(DEFAULT_SHOW_UNTRACKED),
                cache_file,
            },
            Err(_) => {
                // Build the default configuration.
                Config {
                    basedir: homedir,
                    follow_symlinks: DEFAULT_FOLLOW_SYMLINKS,
                    same_filesystem: DEFAULT_SAME_FILESYSTEM,
                    ignored_patterns: vec![],
                    default_cmd: String::from(DEFAULT_CMD),
                    show_untracked: DEFAULT_SHOW_UNTRACKED,
                    cache_file,
                }
            }
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
            if let Some(file) = &self.cache_file {
                remove_file(&file).expect("Failed to delete cache file.");
            }
        }
    }

    /// Returns `true` if this directory entry should be included in scans.
    fn filter(&self, entry: &DirEntry) -> bool {
        if let Some(entry_path) = entry.path().to_str() {
            self.ignored_patterns
                .iter()
                .filter(|p| p != &"")
                .all(|pattern| !entry_path.contains(pattern))
        } else {
            // Skip invalid file name
            false
        }
    }

    /// Walks the configured base directory, looking for git repos.
    fn find_repos(&self) -> Vec<Repo> {
        let mut repos = Vec::new();
        println!(
            "Scanning for git repos under {}; this may take a while...",
            self.basedir.display()
        );
        let walker = WalkDir::new(&self.basedir)
            .follow_links(self.follow_symlinks)
            .same_file_system(self.same_filesystem);
        for entry in walker.into_iter().filter_entry(|e| self.filter(e)) {
            if let Ok(entry) = entry {
                if entry.file_type().is_dir() && entry.file_name() == ".git" {
                    let parent_path = entry
                        .path()
                        .parent()
                        .expect("Could not determine parent.");
                    if let Some(path) = parent_path.to_str() {
                        repos.push(Repo::new(path.to_string()));
                    }
                }
            }
        }
        repos.sort_by_key(|r| r.path());
        repos
    }

    /// Returns boolean indicating if the cache file exists.
    fn has_cache(&self) -> bool {
        self.cache_file.as_ref().map_or(false, |f| f.exists())
    }

    /// Writes the given repo paths to the cache file.
    fn cache_repos(&self, repos: &[Repo]) {
        if let Some(file) = &self.cache_file {
            if !file.exists() {
                if let Some(parent) = &file.parent() {
                    create_dir_all(parent)
                        .expect("Could not create cache directory.")
                }
            }
            let mut f =
                File::create(file).expect("Could not create cache file.");
            for repo in repos.iter() {
                match writeln!(f, "{}", repo.path()) {
                    Ok(_) => (),
                    Err(e) => panic!("Problem writing cache file: {}", e),
                }
            }
        }
    }

    /// Returns the list of repos found in the cache file.
    fn get_cached_repos(&self) -> Vec<Repo> {
        let mut repos = Vec::new();
        if let Some(file) = &self.cache_file {
            if file.exists() {
                let f = File::open(file).expect("Could not open cache file.");
                let reader = BufReader::new(f);
                for line in reader.lines() {
                    if let Ok(repo_path) = line {
                        if !Path::new(&repo_path).exists() {
                            continue;
                        }
                        repos.push(Repo::new(repo_path))
                    }
                }
            }
        }
        repos
    }
}
