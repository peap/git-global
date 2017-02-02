//! Keep track of all your local git repositories.
//!
//! This crate houses the binary and library for the git-global subcommand, a
//! way to query statuses of all your local git repos. The binary can be
//! installed with cargo: `cargo install git-global`.
//!
//! # Usage
//!
//! ```bash
//! $ git global [status]  # get the status of all known repos
//! $ git global scan      # (re)scan home directory to discover git repos
//! $ git global list      # list all known repos
//! $ git global info      # show meta-information about git-global
//! ```

extern crate app_dirs;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate git2;
#[macro_use]
extern crate json;
extern crate walkdir;

#[cfg(test)]
extern crate tempdir;

mod cli;
mod core;
mod errors;
pub mod subcommands;  // Using `pub mod` so we see the docs.

pub use cli::run_from_command_line;
pub use core::{GitGlobalResult, Repo, get_repos};
pub use errors::Result;
pub use errors::GitGlobalError;
