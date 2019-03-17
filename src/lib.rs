//! Keep track of all your git repositories.
//!
//! This crate houses the binary and library for the git-global subcommand, a
//! way to find, query statuses, and gain other insights about all the git repos
//! on your machine. The binary can be installed with cargo: `cargo install
//! git-global`.
//!
//! # Command-line Usage
//!
//! ```bash
//! $ git global [status]  # show `git status -s` for all your git repos
//! $ git global info      # show information about git-global itself
//! $ git global list      # show all git repos git-global knows about
//! $ git global scan      # search your filesystem for git repos and update cache
//! ```
//!
//! # Public Interface
//!
//! The [`Repo`] struct is a git repository that is identified by the full path
//! to its base directory (i.e., not its `.git` directory).
//!
//! The [`GitGlobalConfig`] struct holds a user's git-global configuration
//! information, which merges some default values with values in the `[global]`
//! section of the user's global `.gitconfig` file.
//!
//! A [`GitGlobalResult`] result contains messages added by a subcommand, either
//! about the overall process or about a specific repo, as well as a list of
//! repos. All subcommands expose a `get_results()` function that returns a
//! `GitGlobalResult`.
//!
//! The [`get_repos()`] function returns the list of known repos, performing a
//! scan if necessary.
//!
//! All git-global subcommands are implemented in the [`subcommands`] module.
//!
//! [`Repo`]: struct.Repo.html
//! [`GitGlobalConfig`]: struct.GitGlobalConfig.html
//! [`GitGlobalResult`]: struct.GitGlobalResult.html
//! [`get_repos()`]: fn.get_repos.html
//! [`subcommands`]: subcommands/index.html

extern crate app_dirs;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate dirs;
extern crate git2;
#[macro_use]
extern crate json;
extern crate walkdir;

mod cli;
mod core;
mod errors;
pub mod subcommands; // Using `pub mod` so we see the docs.

pub use cli::run_from_command_line;
pub use core::{get_repos, GitGlobalConfig, GitGlobalResult, Repo};
pub use errors::GitGlobalError;
pub use errors::Result;
