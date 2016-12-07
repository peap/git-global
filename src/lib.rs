//! git-global: git subcommand for working with all git repos on a machine

#[macro_use]
extern crate clap;
extern crate git2;
#[macro_use]
extern crate json;
extern crate walkdir;
extern crate xdg;

mod cli;
mod core;
mod errors;
mod subcommands;

pub use cli::run_from_command_line;
pub use core::{GitGlobalResult, Repo, get_repos};
pub use errors::Result;
pub use errors::GitGlobalError;
