//! Reporting for git-global.

use std::collections::HashMap;
use std::io::Write;

use serde_json::json;

use crate::repo::Repo;

/// A report containing the results of a git-global subcommand.
///
/// Contains overall messages and per-repo messages.
pub struct Report {
    messages: Vec<String>,
    repo_messages: HashMap<Repo, Vec<String>>,
    repos: Vec<Repo>,
    pad_repo_output: bool,
}

impl Report {
    /// Create a new `Report` for the given `Repo`s..
    pub fn new(repos: &[Repo]) -> Report {
        let mut repo_messages: HashMap<Repo, Vec<String>> = HashMap::new();
        for repo in repos {
            repo_messages.insert(repo.clone(), Vec::new());
        }
        Report {
            messages: Vec::new(),
            repos: repos.to_owned(),
            repo_messages,
            pad_repo_output: false,
        }
    }

    /// Declares the desire to separate output when showing per-repo messages.
    ///
    /// Sets flag that indicates a blank line should be inserted between
    /// messages for different repos when printing per-repo output.
    pub fn pad_repo_output(&mut self) {
        self.pad_repo_output = true;
    }

    /// Adds a message that applies to the overall operation.
    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    /// Adds a message that applies to the given repo.
    pub fn add_repo_message(&mut self, repo: &Repo, data_line: String) {
        if let Some(item) = self.repo_messages.get_mut(repo) {
            item.push(data_line)
        }
    }

    /// Writes all result messages to the given writer, as text.
    pub fn print<W: Write>(&self, writer: &mut W) {
        for msg in self.messages.iter() {
            writeln!(writer, "{}", msg).unwrap();
        }
        for repo in self.repos.iter() {
            let messages = self.repo_messages.get(repo).unwrap();
            if !messages.is_empty() {
                writeln!(writer, "{}", repo).unwrap();
                for line in messages.iter().filter(|l| !l.is_empty()) {
                    writeln!(writer, "{}", line).unwrap();
                }
                if self.pad_repo_output {
                    writeln!(writer).unwrap();
                }
            }
        }
    }

    /// Writes all result messages to the given writer, as JSON.
    pub fn print_json<W: Write>(&self, writer: &mut W) {
        let mut repo_messages: HashMap<String, Vec<&String>> = HashMap::new();
        for (repo, messages) in self.repo_messages.iter() {
            let msgs = messages.iter().filter(|l| !l.is_empty());
            repo_messages.insert(repo.path(), msgs.collect());
        }
        let json = json!({
            "error": false,
            "messages": self.messages,
            "repo_messages": repo_messages
        });
        writeln!(writer, "{:#}", json).unwrap();
    }
}
