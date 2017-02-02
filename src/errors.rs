//! Error handling for git-global.

use std::error::Error;
use std::fmt;
use std::io;
use std::result;

/// An error.
#[derive(Debug)]
pub enum GitGlobalError {
    BadSubcommand(String),
    Generic,
}

/// Our `Result` alias with `GitGlobalError` as the error type.
pub type Result<T> = result::Result<T, GitGlobalError>;

impl fmt::Display for GitGlobalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use GitGlobalError::*;
        match *self {
            BadSubcommand(ref cmd) => write!(f, "Unknown subcommand, {}.", cmd),
            Generic => write!(f, "An error occured :(."),
        }
    }
}

impl Error for GitGlobalError {
    fn description(&self) -> &str {
        use GitGlobalError::*;
        match *self {
            BadSubcommand(_) => "unknown subcommand",
            Generic => "an error occurred :(",
        }
    }
}

impl From<io::Error> for GitGlobalError {
    #[allow(unused_variables)]
    fn from(err: io::Error) -> GitGlobalError {
        GitGlobalError::Generic
    }
}
