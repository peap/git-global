//! The command line interface for git-global.

use std::io::{stderr, stdout, Write};

use clap::{App, Arg, SubCommand};

use config::GitGlobalConfig;
use subcommands;

/// Returns the definitive clap::App instance for git-global.
fn get_clap_app<'a, 'b>() -> App<'a, 'b> {
    App::new("git-global")
        .version(crate_version!())
        .author("Eric Petersen <eric@ericpetersen.io>")
        .about("git subcommand for working with all git repos on a machine")
        .arg(
            Arg::with_name("json")
                .long("json")
                .help("Output subcommand results in JSON."),
        )
        .subcommand(
            SubCommand::with_name("info")
                .about("Shows meta-information about git-global"),
        )
        .subcommand(
            SubCommand::with_name("list").about("Lists all known git repos"),
        )
        .subcommand(
            SubCommand::with_name("scan")
                .about("Updates cache of known git repos"),
        )
        .subcommand(
            SubCommand::with_name("status")
                .about("Shows status of all known git repos [the default]"),
        )
}

/// Runs the appropriate git-global subcommand based on command line arguments.
///
/// As the effective binary entry point for `git-global`, prints results to
/// `STDOUT` (or errors to `STDERR~) and returns an exit code.
pub fn run_from_command_line() -> i32 {
    let clap_app = get_clap_app();
    let matches = clap_app.get_matches();
    let config = GitGlobalConfig::new();
    let report = match matches.subcommand_name() {
        Some(cmd) => subcommands::run(cmd, config),
        None => subcommands::run("status", config),
    };
    let use_json = matches.is_present("json");
    match report {
        Ok(rep) => {
            if use_json {
                rep.print_json(&mut stdout());
            } else {
                rep.print(&mut stdout());
            }
            0
        }
        Err(err) => {
            if use_json {
                let json = object! {
                    "error" => true,
                    "message" => format!("{}", err)
                };
                writeln!(&mut stderr(), "{:#}", json).unwrap();
            } else {
                writeln!(&mut stderr(), "{}", err).unwrap();
            }
            1
        }
    }
}
