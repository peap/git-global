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
//! This project's primary goal is produce a useful binary. There's no real
//! intent to provide a good library for other Rust projects to use, so this
//! documentation primarily serves to illustrate how the codebase is structured.
//! If a library use case arises, however, that would be fine.
//!
//! The [`Repo`] struct is a git repository that is identified by the full path
//! to its base directory (i.e., not its `.git` directory).
//!
//! The [`Config`] struct holds a user's git-global configuration information,
//! which usually merges some default values with values in the `[global]`
//! section of the user's global `.gitconfig` file. Provides access to the list
//! of `Repo`s to subcommands via its `get_repos()` method, which populates a
//! cache file, if necessary.
//!
//! A [`SubcommandReport`] contains messages added by a subcommand, either about
//! the overall process or about a specific repo, as well as the list of repos
//! to which the report applies. All git-global subcommands are implemented as
//! submodules in the [`subcommands`] module that expose a `run()` function.
//!
//! The [`run_from_command_line()`] function handles running git-global from the
//! command line and is the effective entry point for the binary.
//!
//! [`Repo`]: struct.Repo.html
//! [`Config`]: struct.Config.html
//! [`SubcommandReport`]: subcommands/struct.SubcommandReport.html
//! [`subcommands`]: subcommands/index.html
//! [`run_from_command_line()`]: fn.run_from_command_line.html

extern crate app_dirs;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate git2;
#[macro_use]
extern crate json;
extern crate walkdir;

mod cli;
mod config;
mod errors;
mod repo;
pub mod subcommands;  // Using `pub mod` so we see the docs.

pub use cli::run_from_command_line;
pub use config::Config;
pub use errors::Result;
pub use errors::GitGlobalError;
pub use repo::Repo;
pub use subcommands::SubcommandReport;
