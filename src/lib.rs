//! git-global: git subcommand for working with all git repos on a machine

extern crate app_dirs;
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
