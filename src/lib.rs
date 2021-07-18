//! Keep track of all the git repositories on your machine.
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
//! # ...
//! $ git global help      # show usage and all subcommands
//! ```
//!
//! # Public Interface
//!
//! The git-global project's primary goal is to produce a useful binary. There's
//! no driving force to provide a very good library for other Rust projects to
//! use, so this documentation primarily serves to illustrate how the codebase
//! is structured. (If a library use-case arises, however, that would be fine.)
//!
//! The [`Repo`] struct is a git repository that is identified by the full path
//! to its base directory (instead of, say, its `.git` directory).
//!
//! The [`Config`] struct holds a user's git-global configuration information,
//! which usually merges some default values with values in the `[global]`
//! section of the user's global `.gitconfig` file. It provides access to the
//! list of known `Repo`s via the `get_repos()` method, which reads from a cache
//! file, populating it for the first time after performing a filesystem scan,
//! if necessary.
//!
//! A [`Report`] contains messages added by a subcommand about the overall
//! results of what it did, as well as messages about the specific `Repo`s to
//! which that subcommand applies. All subcommand modules expose an `execute()`
//! function that takes ownership of a `Config` struct and returns a
//! `Result<Report>`. These subcommands live in the [`subcommands`][subcommands]
//! module.
//!
//! The [`run_from_command_line()`][rfcl] function handles running git-global
//! from the command line and serves as the entry point for the binary.
//!
//! [`Config`]: struct.Config.html
//! [`Repo`]: struct.Repo.html
//! [`Report`]: struct.Report.html
//! [rfcl]: fn.run_from_command_line.html
//! [subcommands]: subcommands/index.html

mod cli;
mod config;
mod errors;
mod repo;
mod report;
pub mod subcommands; // Using `pub mod` so we see the docs.

pub use cli::run_from_command_line;
pub use config::Config;
pub use errors::{GitGlobalError, Result};
pub use repo::Repo;
pub use report::Report;
