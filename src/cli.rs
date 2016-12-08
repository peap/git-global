//! cli: command line parsing for git-global

use std::io::{Write, stderr};

use clap::{Arg, App, SubCommand};

use core::GitGlobalResult;
use errors::GitGlobalError;
use subcommands;

fn get_clap_app<'a, 'b>() -> App<'a, 'b> {
    App::new("git-global")
        .version(crate_version!())
        .author("Eric Petersen <eric@huskers.unl.edu>")
        .about("git subcommand for working with all git repos on a machine")
        .arg(Arg::with_name("json")
            .long("json")
            .help("Output results in JSON."))
        .subcommand(SubCommand::with_name("list")
            .about("lists all git repos on your machine [the default]"))
        .subcommand(SubCommand::with_name("scan")
            .about("update cache of git repos on your machine"))
        .subcommand(SubCommand::with_name("status")
            .about("shows status of all git repos"))
}

/// Entry point for the `git-global` git subcommand. Returns an exit code.
pub fn run_from_command_line() -> i32 {
    let clap_app = get_clap_app();
    let matches = clap_app.get_matches();
    let use_json = matches.is_present("json");
    let results = match matches.subcommand_name() {
        Some("list") => subcommands::list::get_results(),
        Some("scan") => subcommands::scan::get_results(),
        Some("status") => subcommands::status::get_results(),
        Some(cmd) => Err(GitGlobalError::BadSubcommand(cmd.to_string())),
        None => subcommands::list::get_results(),
    };
    match results {
        Ok(res) => show_results(res, use_json),
        Err(err) => show_error(err, use_json),
    }
}

/// Print out the subcommand results.
fn show_results(results: GitGlobalResult, use_json: bool) -> i32 {
    if use_json {
        results.print_json();
    } else {
        results.print();
    }
    0
}

/// Print out an error.
fn show_error(error: GitGlobalError, use_json: bool) -> i32 {
    if use_json {
        let json = object!{
            "error" => true,
            "message" => format!("{}", error)
        };
        writeln!(&mut stderr(), "{:#}", json).expect("failed write to STDERR");;
    } else {
        writeln!(&mut stderr(), "{}", error).expect("failed write to STDERR");
    }
    1
}
