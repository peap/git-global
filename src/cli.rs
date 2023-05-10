//! The command line interface for git-global.

use std::io::{stderr, stdout, Write};

use clap::{app_from_crate, App, Arg, ArgMatches, SubCommand};
use json::object;

use crate::config::Config;
use crate::subcommands;

/// Returns the definitive clap::App instance for git-global.
pub fn get_clap_app<'a>() -> App<'a> {
    app_from_crate!()
        .arg(
            Arg::with_name("json")
                .short('j')
                .long("json")
                .global(true)
                .help("Output subcommand results in JSON."),
        )
        .arg(
            Arg::with_name("untracked")
                .short('u')
                .long("untracked")
                .conflicts_with("nountracked")
                .global(true)
                .help("Show untracked files in output."),
        )
        .arg(
            Arg::with_name("nountracked")
                .short('t')
                .long("nountracked")
                .conflicts_with("untracked")
                .global(true)
                .help("Don't show untracked files in output."),
        )
        .subcommands(
            subcommands::get_subcommands()
                .iter()
                .map(|(cmd, desc)| SubCommand::with_name(cmd).about(*desc)),
        )
}

/// Merge command-line arguments from an ArgMatches object with a Config.
fn merge_args_with_config(config: &mut Config, matches: &ArgMatches) {
    if matches.is_present("untracked") {
        config.show_untracked = true;
    }
    if matches.is_present("nountracked") {
        config.show_untracked = false;
    }
}

/// Runs the appropriate git-global subcommand based on command line arguments.
///
/// As the effective binary entry point for `git-global`, prints results to
/// `STDOUT` (or errors to `STDERR`) and returns an exit code.
pub fn run_from_command_line() -> i32 {
    let clap_app = get_clap_app();
    let matches = clap_app.get_matches();
    let mut config = Config::new();
    merge_args_with_config(&mut config, &matches);
    let report = subcommands::run(matches.subcommand_name(), config);
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
