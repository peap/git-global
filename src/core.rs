//! Core functionality of git-global.
//!
//! Includes the `Repo`, `GitGlobalConfig`, and `GitGlobalResult` structs, and
//! the `get_repos()` function.

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use app_dirs::{AppInfo, AppDataType, app_dir, get_app_dir};
use dirs::home_dir;
use git2;
use walkdir::WalkDir;

const APP: AppInfo = AppInfo { name: "git-global", author: "peap" };
const CACHE_FILE: &'static str = "repos.txt";
const SETTING_BASEDIR: &'static str = "global.basedir";
const SETTING_IGNORED: &'static str = "global.ignore";

/// A git repository, represented by the full path to its base directory.
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Repo {
    path: String,
}

impl Repo {
    pub fn new(path: String) -> Repo {
        Repo {
            path: path,
        }
    }

    /// Returns the full path to the repo as a `String`.
    pub fn path(&self) -> String {
        self.path.clone()
    }

    /// Returns the `git2::Repository` equivalent of this repo.
    pub fn as_git2_repo(&self) -> Option<git2::Repository> {
        git2::Repository::open(&self.path).ok()
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

/// The result of a git-global subcommand.
///
/// Contains overall messages, per-repo messages, and a list of repos.
pub struct GitGlobalResult {
    messages: Vec<String>,
    repos: Vec<Repo>,
    repo_messages: HashMap<Repo, Vec<String>>,
    flag_pad_repo_output: bool,
}

impl GitGlobalResult {
    pub fn new(repos: &Vec<Repo>) -> GitGlobalResult {
        let mut repo_messages: HashMap<Repo, Vec<String>> = HashMap::new();
        for repo in repos {
            repo_messages.insert(repo.clone(), Vec::new());
        }
        GitGlobalResult {
            messages: Vec::new(),
            repos: repos.clone(),
            repo_messages: repo_messages,
            flag_pad_repo_output: false,
        }
    }

    /// Declares desire to separate output when showing per-repo messages.
    ///
    /// Sets flag that indicates a blank line should be inserted between
    /// messages for each repo when showing results output.
    pub fn pad_repo_output(&mut self) {
        self.flag_pad_repo_output = true;
    }

    /// Adds a message that applies to the overall operation.
    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    /// Adds a message that applies to a particular repo.
    pub fn add_repo_message(&mut self, repo: &Repo, data_line: String) {
        match self.repo_messages.get_mut(&repo) {
            Some(item) => item.push(data_line),
            None => (),
        }
    }

    /// Writes all result messages to STDOUT, as text.
    pub fn print(&self) {
        for msg in self.messages.iter() {
            println!("{}", msg);
        }
        for repo in self.repos.iter() {
            let messages = self.repo_messages.get(&repo).unwrap();
            if messages.len() > 0 {
                println!("{}", repo);
                for line in messages.iter().filter(|l| *l != "") {
                    println!("{}", line);
                }
                if self.flag_pad_repo_output {
                    println!();
                }
            }
        }
    }

    /// Writes all result messages to STDOUT, as JSON.
    pub fn print_json(&self) {
        let mut json = object!{
            "error" => false,
            "messages" => array![],
            "repo_messages" => object!{}
        };
        for msg in self.messages.iter() {
            json["results"]["messages"]
                .push(msg.to_string())
                .expect("Failing pushing message to JSON messages array.");
        }
        for (repo, messages) in self.repo_messages.iter() {
            json["repo_messages"][&repo.path] = array![];
            if messages.len() > 0 {
                for line in messages.iter().filter(|l| *l != "") {
                    json["repo_messages"][&repo.path]
                        .push(line.to_string())
                        .expect("Failed pushing line to JSON repo-messages array.");
                }
            }
        }
        println!("{:#}", json);
    }
}

/// A container for git-global configuration options.
pub struct GitGlobalConfig {
    pub basedir: String,
    pub ignored_patterns: Vec<String>,
    pub cache_file: PathBuf,
}

impl GitGlobalConfig {
    pub fn new() -> GitGlobalConfig {
        let home_dir = home_dir()
            .expect("Could not determine home directory.")
            .to_str()
            .expect("Could not convert home directory path to string.")
            .to_string();
        let (basedir, patterns) = match git2::Config::open_default() {
            Ok(config) => {
                (config.get_string(SETTING_BASEDIR).unwrap_or(home_dir),
                 config.get_string(SETTING_IGNORED)
                     .unwrap_or(String::new())
                     .split(",")
                     .map(|p| p.trim().to_string())
                     .collect())
            }
            Err(_) => (home_dir, Vec::new()),
        };
        let cache_file = match get_app_dir(AppDataType::UserCache, &APP, "cache") {
            Ok(mut dir) => {
                dir.push(CACHE_FILE);
                dir
            }
            Err(_) => panic!("TODO: work without XDG"),
        };
        GitGlobalConfig {
            basedir: basedir,
            ignored_patterns: patterns,
            cache_file: cache_file,
        }
    }

//    /// Returns `true` if this directory entry should be included in scans.
//    fn filter(&self, entry: &DirEntry) -> bool {
//        let entry_path = entry.path().to_str().expect("DirEntry without path.");
//        self.ignored_patterns
//            .iter()
//            .fold(true, |acc, pattern| acc && !entry_path.contains(pattern))
//    }

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
        let mut f = File::create(&self.cache_file).expect("Could not create cache file.");
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
            let f = File::open(&self.cache_file).expect("Could not open cache file.");
            let reader = BufReader::new(f);
            for line in reader.lines() {
                match line {
                    Ok(repo_path) => repos.push(Repo::new(repo_path)),
                    Err(_) => (),  // TODO: handle errors
                }
            }
        }
        repos
    }
}

/// Walks the configured base directory, looking for git repos.
pub fn find_repos() -> Vec<Repo> {
    let mut repos = Vec::new();
    let user_config = GitGlobalConfig::new();
    let basedir = &user_config.basedir;
    
    println!("Scanning for git repos under {}; this may take a while...",
             basedir);
    for entry in WalkDir::new(basedir) {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_dir() && entry.file_name() == ".git" {
                    let parent_path = entry.path().parent().expect("Could not determine parent.");
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
pub fn cache_repos(repos: &Vec<Repo>) {
    let user_config = GitGlobalConfig::new();
    user_config.cache_repos(repos);
}

/// Returns all known git repos, populating the cache first, if necessary.
pub fn get_repos() -> Vec<Repo> {
    let user_config = GitGlobalConfig::new();
    if !user_config.has_cache() {
        let repos = find_repos();
        cache_repos(&repos);
        repos
    } else {
        user_config.get_cached_repos()
    }
}
