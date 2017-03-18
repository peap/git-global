//! Subcommand implementations and common message handling.

use std::collections::HashMap;

use repo::Repo;

pub mod info;
pub mod list;
pub mod scan;
pub mod status;

// pub trait Subcommand {
//     fn run(config: &Config) -> SubcommandMessages {
//         SubcommandMessages::new()
//     }
// }

/// The result of a git-global subcommand.
///
/// Contains overall messages, per-repo messages, and a list of repos.
pub struct Report {
    messages: Vec<String>,
    repos: Vec<Repo>,
    repo_messages: HashMap<Repo, Vec<String>>,
    flag_pad_repo_output: bool,
}

impl Report {
    pub fn new(repos: &Vec<Repo>) -> Report {
        let mut repo_messages: HashMap<Repo, Vec<String>> = HashMap::new();
        for repo in repos {
            repo_messages.insert(repo.clone(), Vec::new());
        }
        Report {
            messages: Vec::new(),
            repos: repos.clone(),
            repo_messages: repo_messages,
            flag_pad_repo_output: false,
        }
    }

    pub fn get_repos(&self) -> Vec<&Repo> {
        self.repos.iter().collect()
    }

    /// Declares desire to separate output when showing per-repo messages.
    ///
    /// Sets flag that indicates a blank line should be inserted between
    /// messages for each repo when showing report output.
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

    /// Writes full report to STDOUT, as text.
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

    /// Writes full report to STDOUT, as JSON.
    pub fn print_json(&self) {
        let mut json = object!{
            "error" => false,
            "messages" => array![],
            "repo_messages" => object!{}
        };
        for msg in self.messages.iter() {
            json["messages"]
                .push(msg.to_string())
                .expect("Failing pushing message to JSON messages array.");
        }
        for (repo, messages) in self.repo_messages.iter() {
            if messages.len() > 0 {
                json["repo_messages"][repo.get_path_as_string()] = array![];
                for line in messages.iter().filter(|l| *l != "") {
                    json["repo_messages"][repo.get_path_as_string()]
                        .push(line.to_string())
                        .expect("Failed pushing line to JSON repo-messages array.");
                }
            }
        }
        println!("{:#}", json);
    }

}
