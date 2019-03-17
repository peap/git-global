//! Core functionality of git-global.
//!
//! Includes the `Repo`, and `GitGlobalResult` structs, and the `get_repos()`,
//! `cache_repos()`, and `find_repos()` functions.

use std::collections::HashMap;

use walkdir::WalkDir;

use config::GitGlobalConfig;
use repo::Repo;

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
        let mut json = object! {
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
            json["repo_messages"][repo.path()] = array![];
            if messages.len() > 0 {
                for line in messages.iter().filter(|l| *l != "") {
                    json["repo_messages"][repo.path()]
                        .push(line.to_string())
                        .expect(
                            "Failed pushing line to JSON repo-messages array.",
                        );
                }
            }
        }
        println!("{:#}", json);
    }
}

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
