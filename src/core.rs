//! core: core functionality for the `git-global` command

use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use git2;
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

    pub fn as_git2_repo(&self) -> Option<git2::Repository> {
        git2::Repository::open(&self.path).ok()
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

/// The result of a `git-global` subcommand.
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

    pub fn pad_repo_output(&mut self) {
        self.flag_pad_repo_output = true;
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    pub fn add_repo_message(&mut self, repo: &Repo, data_line: String) {
        match self.repo_messages.get_mut(&repo) {
            Some(item) => item.push(data_line),
            None => (),
        }
    }

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
        let entry_path = entry.path().to_str().expect("DirEntry without path.");
        self.ignored_patterns
            .iter()
            .fold(true, |acc, pattern| acc && !entry_path.contains(pattern))
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
