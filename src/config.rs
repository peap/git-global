//! Configuration of git-global.
//!
//! Exports the `Config` struct, which defines the base path for finding git
//! repos on the machine, path patterns to ignore when scanning for repos, the
//! location of a cache file, and other config options for running git-global.

use std::env;
use std::fs::{File, create_dir_all, remove_file};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use directories::{ProjectDirs, UserDirs};
use walkdir::{DirEntry, WalkDir};

use crate::repo::Repo;

const QUALIFIER: &str = "";
const ORGANIZATION: &str = "peap";
const APPLICATION: &str = "git-global";
const CACHE_FILE: &str = "repos.txt";
const IGNORED_REPOS_FILE: &str = "ignored.txt";

const DEFAULT_CMD: &str = "status";
const DEFAULT_FOLLOW_SYMLINKS: bool = true;
const DEFAULT_SAME_FILESYSTEM: bool = cfg!(any(unix, windows));
const DEFAULT_VERBOSE: bool = false;
const DEFAULT_SHOW_UNTRACKED: bool = true;

const SETTING_BASEDIR: &str = "global.basedir";
const SETTING_FOLLOW_SYMLINKS: &str = "global.follow-symlinks";
const SETTING_SAME_FILESYSTEM: &str = "global.same-filesystem";
const SETTING_IGNORE: &str = "global.ignore";
const SETTING_DEFAULT_CMD: &str = "global.default-cmd";
const SETTING_SHOW_UNTRACKED: &str = "global.show-untracked";
const SETTING_VERBOSE: &str = "global.verbose";

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

    /// Whether to enable verbose mode.
    ///
    /// Default: false
    pub verbose: bool,

    /// Whether to show untracked files in output.
    ///
    /// Default: true
    pub show_untracked: bool,

    /// Optional path to a cache file for git-global's usage.
    ///
    /// Default: `repos.txt` in the user's XDG cache directory, if we understand
    /// XDG for the host system.
    pub cache_file: Option<PathBuf>,

    /// Optional path to our manpage, regardless of whether it's installed.
    ///
    /// Default: `git-global.1` in the relevant manpages directory, if we
    /// understand where that should be for the host system.
    pub manpage_file: Option<PathBuf>,

    /// Optional path to a file listing ignored repos.
    ///
    /// Default: `ignored.txt` in the user's XDG cache directory.
    pub ignored_repos_file: Option<PathBuf>,
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
        let project_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION);
        let cache_file = project_dirs
            .as_ref()
            .map(|pd| pd.cache_dir().join(CACHE_FILE));
        let ignored_repos_file = project_dirs
            .as_ref()
            .map(|pd| pd.cache_dir().join(IGNORED_REPOS_FILE));
        let manpage_file = match env::consts::OS {
            // Consider ~/.local/share/man/man1/, too.
            "linux" => Some(PathBuf::from("/usr/share/man/man1/git-global.1")),
            "macos" => Some(PathBuf::from("/usr/share/man/man1/git-global.1")),
            "windows" => env::var("MSYSTEM").ok().and_then(|val| {
                (val == "MINGW64").then(|| {
                    PathBuf::from("/mingw64/share/doc/git-doc/git-global.html")
                })
            }),
            _ => None,
        };
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
                verbose: cfg
                    .get_bool(SETTING_VERBOSE)
                    .unwrap_or(DEFAULT_VERBOSE),
                show_untracked: cfg
                    .get_bool(SETTING_SHOW_UNTRACKED)
                    .unwrap_or(DEFAULT_SHOW_UNTRACKED),
                cache_file,
                manpage_file,
                ignored_repos_file,
            },
            Err(_) => {
                // Build the default configuration.
                Config {
                    basedir: homedir,
                    follow_symlinks: DEFAULT_FOLLOW_SYMLINKS,
                    same_filesystem: DEFAULT_SAME_FILESYSTEM,
                    ignored_patterns: vec![],
                    default_cmd: String::from(DEFAULT_CMD),
                    verbose: DEFAULT_VERBOSE,
                    show_untracked: DEFAULT_SHOW_UNTRACKED,
                    cache_file,
                    manpage_file,
                    ignored_repos_file,
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
        if self.has_cache()
            && let Some(file) = &self.cache_file
        {
            remove_file(file).expect("Failed to delete cache file.");
        }
    }

    /// Returns `true` if this directory entry should be included in scans.
    fn filter(&self, entry: &DirEntry, ignored_repos: &[String]) -> bool {
        let Some(entry_path) = entry.path().to_str() else {
            // Skip invalid file name
            return false;
        };

        // Check against ignored_patterns (from global.ignore config)
        let matches_pattern = self
            .ignored_patterns
            .iter()
            .filter(|p| !p.is_empty())
            .any(|pattern| entry_path.contains(pattern));
        if matches_pattern {
            return false;
        }

        // Check against ignored repos list (from ignore subcommand)
        // Get canonical path for symlink handling
        let canonical_path = entry
            .path()
            .canonicalize()
            .ok()
            .and_then(|p| p.to_str().map(String::from));

        let is_ignored = ignored_repos.iter().any(|ignored_path| {
            // Check if entry is the ignored path or under it
            entry_path == ignored_path
                || entry_path.starts_with(&format!("{}/", ignored_path))
                // Also check canonical path
                || canonical_path.as_ref().is_some_and(|cp| {
                    cp == ignored_path || cp.starts_with(&format!("{}/", ignored_path))
                })
        });

        !is_ignored
    }

    /// Walks the configured base directory, looking for git repos.
    fn find_repos(&self) -> Vec<Repo> {
        let mut repos = Vec::new();
        let ignored_repos = self.get_ignored_repos();
        println!(
            "Scanning for git repos under {}; this may take a while...",
            self.basedir.display()
        );
        let mut n_dirs = 0;
        let walker = WalkDir::new(&self.basedir)
            .follow_links(self.follow_symlinks)
            .same_file_system(self.same_filesystem);
        for entry in walker
            .into_iter()
            .filter_entry(|e| self.filter(e, &ignored_repos))
            .flatten()
        {
            if entry.file_type().is_dir() {
                n_dirs += 1;
                if entry.file_name() == ".git" {
                    let parent_path = entry
                        .path()
                        .parent()
                        .expect("Could not determine parent.");
                    // Validate it's actually a valid git repo before adding
                    if git2::Repository::open(parent_path).is_ok() {
                        if let Some(path) = parent_path.to_str() {
                            repos.push(Repo::new(path.to_string()));
                        }
                    }
                }
                if self.verbose
                    && let Some(size) = termsize::get()
                {
                    let prefix = format!(
                        "\r... found {} repos; scanning directory #{}: ",
                        repos.len(),
                        n_dirs
                    );
                    let width = size.cols as usize - prefix.len() - 1;
                    let mut cur_path =
                        String::from(entry.path().to_str().unwrap());
                    let byte_width = match cur_path.char_indices().nth(width) {
                        None => &cur_path,
                        Some((idx, _)) => &cur_path[..idx],
                    }
                    .len();
                    cur_path.truncate(byte_width);
                    print!("{}{:<width$}", prefix, cur_path);
                }
            }
        }
        if self.verbose {
            println!();
        }
        repos.sort_by_key(|r| r.path());
        repos
    }

    /// Returns boolean indicating if the cache file exists.
    fn has_cache(&self) -> bool {
        self.cache_file.as_ref().is_some_and(|f| f.exists())
    }

    /// Writes the given repo paths to the cache file.
    fn cache_repos(&self, repos: &[Repo]) {
        if let Some(file) = &self.cache_file {
            if !file.exists()
                && let Some(parent) = &file.parent()
            {
                create_dir_all(parent)
                    .expect("Could not create cache directory.")
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

    /// Returns the list of repos found in the cache file, excluding ignored repos.
    fn get_cached_repos(&self) -> Vec<Repo> {
        let ignored = self.get_ignored_repos();
        let mut repos = Vec::new();
        if let Some(file) = &self.cache_file
            && file.exists()
        {
            let f = File::open(file).expect("Could not open cache file.");
            let reader = BufReader::new(f);
            for repo_path in reader.lines().map_while(Result::ok) {
                let path = Path::new(&repo_path);
                if !path.exists() {
                    continue;
                }
                // Get canonical path if possible (resolves symlinks)
                let canonical_path = path
                    .canonicalize()
                    .ok()
                    .and_then(|p| p.to_str().map(String::from));
                // Check if repo matches any ignored path (exact match or prefix)
                // Check both original and canonical paths since symlinks inside
                // a path might resolve outside the ignored prefix
                let is_ignored = ignored.iter().any(|ignored_path| {
                    let prefix = format!("{}/", ignored_path);
                    // Check original path
                    repo_path == *ignored_path
                        || repo_path.starts_with(&prefix)
                        // Check canonical path if available
                        || canonical_path.as_ref().is_some_and(|cp| {
                            cp == ignored_path || cp.starts_with(&prefix)
                        })
                });
                if is_ignored {
                    continue;
                }
                repos.push(Repo::new(repo_path))
            }
        }
        repos
    }

    /// Returns the list of ignored repo paths.
    pub fn get_ignored_repos(&self) -> Vec<String> {
        let mut ignored = Vec::new();
        if let Some(file) = &self.ignored_repos_file
            && file.exists()
        {
            let f = File::open(file).expect("Could not open ignored repos file.");
            let reader = BufReader::new(f);
            for repo_path in reader.lines().map_while(Result::ok) {
                ignored.push(repo_path);
            }
        }
        ignored
    }

    /// Adds a path to the ignored repos file.
    /// Can be a git repo or a directory prefix (all repos under it will be ignored).
    pub fn ignore_repo(&self, repo_path: &str) -> Result<(), String> {
        // Canonicalize the path to ensure consistency
        let canonical = Path::new(repo_path)
            .canonicalize()
            .map_err(|e| format!("Invalid path '{}': {}", repo_path, e))?;
        let canonical_str = canonical
            .to_str()
            .ok_or_else(|| format!("Path '{}' contains invalid UTF-8", repo_path))?;

        // Check if already ignored
        let ignored = self.get_ignored_repos();
        if ignored.contains(&canonical_str.to_string()) {
            return Err(format!("'{}' is already ignored", canonical_str));
        }

        // Add to ignored file
        if let Some(file) = &self.ignored_repos_file {
            if !file.exists() {
                if let Some(parent) = file.parent() {
                    create_dir_all(parent)
                        .map_err(|e| format!("Could not create directory: {}", e))?;
                }
            }
            let mut f = File::options()
                .create(true)
                .append(true)
                .open(file)
                .map_err(|e| format!("Could not open ignored repos file: {}", e))?;
            writeln!(f, "{}", canonical_str)
                .map_err(|e| format!("Could not write to ignored repos file: {}", e))?;
            Ok(())
        } else {
            Err("No ignored repos file configured".to_string())
        }
    }
}
