//! core: core functionality for the `git-global` command

use std::collections::HashMap;
use std::env;
use std::fmt;

use git2::{Config};
use walkdir::{DirEntry, WalkDir, WalkDirIterator};

const SETTING_BASEDIR: &'static str = "global.basedir";
const SETTING_IGNORED: &'static str = "global.ignore";

/// A git repo.
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Repo {
    path: String,
}

impl Repo {
    pub fn new(path: String) -> Repo {
        Repo { path: path }
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
    data: HashMap<Repo, Vec<String>>,
}

impl GitGlobalResult {
    pub fn new(repos: &Vec<Repo>) -> GitGlobalResult {
        let mut data: HashMap<Repo, Vec<String>> = HashMap::new();
        for repo in repos {
            data.insert(repo.clone(), Vec::new());
        }
        GitGlobalResult { data: data }
    }

    pub fn append(&mut self, repo: &Repo, data_line: String) {
        match self.data.get_mut(&repo) {
            Some(item) => item.push(data_line),
            None => ()
        }
    }

    pub fn print(&self) {
        for (repo, data) in self.data.iter() {
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
            json["results"].push(object!{
                "path" => repo.path(),
                "data" => array![]
            }).expect("Failed pushing data to JSON results array.");
            for line in data {
                json["results"]["data"].push(line.to_string())
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
}

impl GitGlobalConfig {
    fn new() -> GitGlobalConfig {
        let home_dir = env::home_dir()
                           .expect("Could not determine home directory!")
                           .to_str().unwrap().to_string();
        let (basedir, patterns) = match Config::open_default() {
            Ok(config) => (
                    config.get_string(SETTING_BASEDIR).unwrap_or(home_dir),
                    config.get_string(SETTING_IGNORED).unwrap_or(String::new())
                        .split(",").map(|p| p.trim().to_string()).collect(),
            ),
            Err(_) => (
                home_dir,
                Vec::new(),
            ),
        };
        GitGlobalConfig {
            basedir: basedir,
            ignored_patterns: patterns,
        }
    }

    fn filter(&self, entry: &DirEntry) -> bool {
        self.ignored_patterns.iter().fold(true, |acc, pattern| {
            acc && !entry.path().to_str().unwrap().contains(pattern)
        })
        // !(e.path().to_str().unwrap().contains(".cargo"))
    }
}

/// Scan the machine for git repos.
pub fn get_repos() -> Vec<Repo> {
    let mut repos = Vec::new();
    let user_config = GitGlobalConfig::new();
    let walker = WalkDir::new(&user_config.basedir).into_iter();
    for entry in walker.filter_entry(|e| user_config.filter(e) ) {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_dir() && entry.file_name() == ".git" {
                    let parent_path = entry.path().parent().unwrap();
                    match parent_path.to_str() {
                        Some(path) => {
                            repos.push(Repo::new(path.to_string()));
                        }
                        None => ()
                    }
                }
            }
            Err(_) => (),
        }
    }
    repos
}
