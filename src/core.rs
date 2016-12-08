//! core: core functionality for the `git-global` command

use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use git2::Config;
use walkdir::{DirEntry, WalkDir, WalkDirIterator};
use xdg;

const CACHE_FILE: &'static str = "repos.txt";
const CACHE_PREFIX: &'static str = "git-global";
const SETTING_BASEDIR: &'static str = "global.basedir";
const SETTING_IGNORED: &'static str = "global.ignore";

/// A git repo.
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

    pub fn path(&self) -> String {
        self.path.clone()
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

/// The result of a `git-global` subcommand.
pub struct GitGlobalResult {
    repos: Vec<Repo>,
    data: HashMap<Repo, Vec<String>>,
}

impl GitGlobalResult {
    pub fn new(repos: &Vec<Repo>) -> GitGlobalResult {
        let mut data: HashMap<Repo, Vec<String>> = HashMap::new();
        for repo in repos {
            data.insert(repo.clone(), Vec::new());
        }
        GitGlobalResult {
            repos: repos.clone(),
            data: data,
        }
    }

    pub fn append(&mut self, repo: &Repo, data_line: String) {
        match self.data.get_mut(&repo) {
            Some(item) => item.push(data_line),
            None => (),
        }
    }

    pub fn print(&self) {
        for repo in self.repos.iter() {
            let data = self.data.get(&repo).unwrap();
            println!("{}", repo);
            for line in data {
                println!("{}", line);
            }
        }
    }

    pub fn print_json(&self) {
        let mut json = object!{
            "error" => false,
            "results" => array![]
        };
        for (repo, data) in self.data.iter() {
            json["results"]
                .push(object!{
                    "path" => repo.path(),
                    "data" => array![]
                })
                .expect("Failed pushing data to JSON results array.");
            for line in data {
                json["results"]["data"]
                    .push(line.to_string())
                    .expect("Failed pushing data line to JSON data array.");
            }
        }
        println!("{:#}", json);
    }
}

/// Container for `git-global` configuration options.
struct GitGlobalConfig {
    basedir: String,
    ignored_patterns: Vec<String>,
    cache_dir: xdg::BaseDirectories,
}

impl GitGlobalConfig {
    fn new() -> GitGlobalConfig {
        let home_dir = env::home_dir()
            .expect("Could not determine home directory!")
            .to_str()
            .unwrap()
            .to_string();
        let (basedir, patterns) = match Config::open_default() {
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
        let cache_dir = match xdg::BaseDirectories::with_prefix(CACHE_PREFIX) {
            Ok(dir) => dir,
            Err(_) => panic!("TODO: work without XDG"),
        };
        GitGlobalConfig {
            basedir: basedir,
            ignored_patterns: patterns,
            cache_dir: cache_dir,
        }
    }

    fn filter(&self, entry: &DirEntry) -> bool {
        self.ignored_patterns.iter().fold(true, |acc, pattern| {
            acc && !entry.path().to_str().unwrap().contains(pattern)
        })
    }

    fn has_cache(&self) -> bool {
        match self.cache_dir.find_cache_file(CACHE_FILE) {
            Some(path_buf) => path_buf.as_path().exists(),
            None => false,
        }
    }

    fn cache_repos(&self, repos: &Vec<Repo>) {
        match self.cache_dir.place_cache_file(CACHE_FILE) {
            Ok(path_buf) => {
                let mut f = File::create(path_buf).unwrap();
                for repo in repos.iter() {
                    match writeln!(f, "{}", repo.path()) {
                        Ok(_) => (),
                        Err(e) => panic!("Problem writing cache file: {}", e),
                    }
                }
            }
            Err(e) => panic!("Unable to cache repos: {}", e),
        }
    }

    fn get_cached_repos(&self) -> Vec<Repo> {
        let mut repos = Vec::new();
        match self.cache_dir.find_cache_file(CACHE_FILE) {
            Some(path_buf) => {
                if path_buf.as_path().exists() {
                    let f = File::open(path_buf).unwrap();
                    let reader = BufReader::new(f);
                    for line in reader.lines() {
                        match line {
                            Ok(repo_path) => repos.push(Repo::new(repo_path)),
                            Err(_) => (),  // TODO: handle errors
                        }
                    }
                }
            }
            None => (),  // TODO: handle errors
        }
        repos
    }
}

/// Scans the machine for git repos, taking git-global config into account.
pub fn find_repos() -> Vec<Repo> {
    let mut repos = Vec::new();
    let user_config = GitGlobalConfig::new();
    let basedir = &user_config.basedir;
    let walker = WalkDir::new(basedir).into_iter();
    println!("Scanning for git repos under {}; this may take a while...",
             basedir);
    for entry in walker.filter_entry(|e| user_config.filter(e)) {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_dir() && entry.file_name() == ".git" {
                    let parent_path = entry.path().parent().unwrap();
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

/// Caches repo paths to disk, in the XDG cache directory for "git-global".
pub fn cache_repos(repos: &Vec<Repo>) {
    let user_config = GitGlobalConfig::new();
    user_config.cache_repos(repos);
}

/// Loads cached repo paths from disk.
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
