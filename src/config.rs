//! Configuration handling for git-global, exposed via the `Config` struct.

use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::str::FromStr;

use app_dirs::{AppDataType, AppInfo, app_dir, get_app_dir};
use git2;
use walkdir::{DirEntry, WalkDir, WalkDirIterator};

use repo::Repo;

const APP_INFO: AppInfo = AppInfo { name: "git-global", author: "peap" };
const CACHE_FILENAME: &'static str = "repos.txt";
const SETTING_IGNORED: &'static str = "global.ignore";
const SETTING_SCAN_ROOT: &'static str = "global.scanroot";

#[derive(Hash)]
/// Container for git-global settings, and interface to the list of repos.
pub struct Config {
    /// Filename of the cache file (but not the full path).
    pub cache_filename: String,

    /// List of filename patterns to skip when scanning for repos.
    pub ignored_patterns: Vec<String>,

    /// Root directory for performing scans; defaults to user's home directory.
    pub scan_root: PathBuf,
}

impl Config {

    // ============
    // Constructors
    // ============

    /// Returns a new `Config` instance with default values.
    ///
    /// Default values are:
    ///
    /// * `cache_filename`: `repos.txt`
    /// * `ignored_patterns`: `['.cargo']`
    /// * `scan_root`: your home directory
    ///
    /// So, by default, git-global will look for git repos in your home
    /// directory, ignoring any paths that include `.cargo`, and caching the
    /// list of repos in a file called `repos.txt` in your OS-specific cache
    /// directory.
    pub fn new() -> Config {
        Config {
            cache_filename: Config::get_default_cache_filename(),
            ignored_patterns: Config::get_default_ignored_patterns(),
            scan_root: Config::get_default_scan_root(),
        }
    }

    /// Returns a new `Config` instance with git config values overriding defaults.
    ///
    /// The following git global configuration settings are read:
    ///
    /// * `global.ignore`: comma-separated list of filename patterns to ignore
    ///                    while scanning
    /// * `global.scanroot`: full path from which to scan for git repos
    pub fn from_gitconfig() -> Config {
        let ignored = Config::load_setting(SETTING_IGNORED)
                      .map(|s| {
                          s.split(",").map(|p| p.trim().to_string()).collect()
                       })
                      .unwrap_or(Config::get_default_ignored_patterns());
        let scan_root = Config::load_setting(SETTING_SCAN_ROOT)
                       .map(|s| PathBuf::from(s))
                       .unwrap_or(Config::get_default_scan_root());
        Config {
            cache_filename: Config::get_default_cache_filename(),
            ignored_patterns: ignored,
            scan_root: scan_root,
        }
    }

    // ===================
    // Constructor helpers
    // ===================

    /// Tries to load a setting from the user's global git configuration file.
    fn load_setting(setting: &str) -> Option<String> {
        git2::Config::open_default()
            .map(|conf| conf.get_string(setting))
            .and_then(|s| s)
            .ok()
    }

    /// Returns the default name of the cache file.
    fn get_default_cache_filename() -> String {
        CACHE_FILENAME.to_string()
    }

    /// Returns the default list of filename patterns to ignore.
    fn get_default_ignored_patterns() -> Vec<String> {
        // TODO: Include some OS-specific defaults, like `Library` on macOS?
        vec![String::from(".cargo")]
    }

    /// Returns the default path for performing scans.
    ///
    /// Tries to return the user's home directory, falling back to the
    /// filesystem root. Assumes that a `C:` drive exists on Windows and that a
    /// `/` exists on all other operating systems.
    fn get_default_scan_root() -> PathBuf {
        if let Some(home_dir) = env::home_dir() {
            home_dir
        } else {
            match env::consts::OS {
                "windows" => PathBuf::from("C:\\"),
                _ => PathBuf::from("/"),
            }
        }
    }

    // ==============
    // Repo Discovery
    // ==============

    /// Returns `true` if this directory entry should be searched for repos.
    fn filter(&self, entry: &DirEntry) -> bool {
        let entry_path = entry.path().to_str().expect("DirEntry without path.");
        self.ignored_patterns
            .iter()
            .fold(true, |acc, pattern| acc && !entry_path.contains(pattern))
    }

    /// Walks the configured root directory, looking for git repos.
    fn find_repos(&self) -> Vec<Repo> {
        let mut repos = Vec::new();
        let scan_root = &self.scan_root;
        let walker = WalkDir::new(scan_root).into_iter();
        println!("Scanning for git repos under {}; this may take a while...",
                 scan_root.display());
        for entry in walker.filter_entry(|e| self.filter(e)) {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_dir() && entry.file_name() == ".git" {
                        let parent_path = entry.path().parent()
                                          .expect("Could not determine parent.");
                        repos.push(Repo::new(parent_path.to_path_buf()));
                    }
                }
                Err(_) => (),
            }
        }
        repos.sort_by(|a, b| a.get_path().cmp(&b.get_path()));
        repos
    }

    // =======
    // Caching
    // =======

    /// Returns the path to the cache file for this particular configuration.
    ///
    /// The cache file is tied to the configuration values so that the cache is
    /// rebuilt if any settings change. This also allows us to run the `scan`
    /// subcommand in the test suite without changing a global cache, since the
    /// configuration will be different.
    ///
    /// # Panics
    ///
    /// If `app_dirs` fails to find a canonical user-cache directory, we panic.
    ///
    /// TODO: Don't panic.
    pub fn get_cache_file(&self) -> PathBuf {
        match get_app_dir(AppDataType::UserCache, &APP_INFO, "cache") {
            Ok(mut dir) => {
                dir.push(&self.cache_filename);
                dir
            }
            Err(_) => panic!("TODO: work without XDG"),
        }
    }

    /// Returns the hash value representing these configuration options.
    ///
    /// Note: DefaultHasher values are not guaranteed to be the same between
    /// Rust releases, so cache files might go unused and need to be rebuilt
    /// when a new Rust version is used to build the git-global binary. One
    /// solution here would be to use a concrete hasher, like SipHasher (which
    /// was deprecated in Rust 1.13), or something in another crate.
    fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    /// Returns a boolean indicating if the cache file is valid.
    fn has_valid_cache_file(&self) -> bool {
        let cache_file = self.get_cache_file();
        if cache_file.exists() {
            let f = fs::File::open(cache_file).expect("Could not open cache file.");
            let reader = BufReader::new(f);
            if let Some(line) = reader.lines().next() {
                match line {
                    Ok(hash_string) => {
                        if let Ok(hash_val) = u64::from_str(&hash_string) {
                            // got a numeric hash we can compare to current hash
                            hash_val == self.get_hash()
                        } else {
                            // non-numeric first line
                            false
                        }
                    }
                    // error reading file, so...
                    Err(_) => false,
                }
            } else {
                // empty file
                false
            }
        } else {
            // file doesn't exist
            false
        }
    }

    /// Returns the list of repos found in the cache file.
    fn get_repos_from_cache(&self) -> Vec<Repo> {
        let mut repos = Vec::new();
        let cache_file = self.get_cache_file();
        if cache_file.exists() {
            let f = fs::File::open(cache_file).expect("Could not open cache file.");
            let reader = BufReader::new(f);
            // skip the first line, which is the hash value
            for line in reader.lines().skip(1) {
                match line {
                    Ok(repo_path) => repos.push(Repo::new(PathBuf::from(repo_path))),
                    Err(_) => (),  // TODO: handle errors
                }
            }
        }
        repos
    }

    /// Rescans the configured root directory and updates the cache file.
    ///
    /// # Panics
    ///
    /// Panics if we're not able to create the cache directory or file, or write
    /// lines to the cache file.
    pub fn update_cache(&self) {
        let repos = self.find_repos();
        if !self.get_cache_file().exists() {
            // Try to create the cache directory if the cache *file* doesn't
            // exist; app_dir() creates the directory if necessary, but also
            // handles an existing directory just fine.
            match app_dir(AppDataType::UserCache, &APP_INFO, "cache") {
                Ok(_) => (),
                Err(e) => panic!("Could not create cache directory: {}", e),
            }
        }
        let mut f = fs::File::create(self.get_cache_file())
                    .expect("Could not create cache file.");
        // Write this configuration's hash to the cache file, so we can decide
        // whether to invalidate it later.
        match writeln!(f, "{}", self.get_hash()) {
            Ok(_) => (),
            Err(e) => panic!("Problem writing hash to cache file: {}", e),
        };
        // Write each repo's path to the cache file.
        for repo in repos.iter() {
            match writeln!(f, "{}", repo.get_path_as_string()) {
                Ok(_) => (),
                Err(e) => panic!("Problem writing cache file: {}", e),
            }
        }
    }

    /// Returns all known repos under scan root.
    ///
    /// First checks the cache file, then (re)populates it, if necessary.
    pub fn get_repos(&self) -> Vec<Repo> {
        if !self.has_valid_cache_file() {
            self.update_cache();
        }
        self.get_repos_from_cache()
    }

}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;

    use super::Config;

    #[test]
    fn test_default_config_creation_has_home_dir_as_scan_root() {
        let config = Config::new();
        match env::home_dir() {
            Some(home) => {
                assert_eq!(&home, &config.scan_root);
            }
            None => {
                let root_dir = match env::consts::OS {
                    "windows" => PathBuf::from("C:\\"),
                    _ => PathBuf::from("/"),
                };
                assert_eq!(&root_dir, &config.scan_root);
            }
        }
    }
}
