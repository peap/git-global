//! Keep track of all your git repositories.
//!
//! This crate houses the binary and library for the git-global subcommand, a
//! way to query statuses of all your git repos. The binary can be installed
//! with cargo: `cargo install git-global`.
//!
//! # Usage
//!
//! ```bash
//! $ git global [status]  # show `git status` for all your git repos
//! $ git global info      # show information about git-global itself
//! $ git global list      # show all git repos git-global knows about
//! $ git global scan      # search your filesystem for git repos and update cache
//! ```

extern crate app_dirs;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate git2;
#[macro_use]
extern crate json;
extern crate walkdir;

mod cli;
mod core;
mod errors;
pub mod subcommands;  // Using `pub mod` so we see the docs.

pub use cli::run_from_command_line;
pub use core::{GitGlobalResult, Repo, get_repos};
pub use errors::Result;
pub use errors::GitGlobalError;
