//! The command line interface for git-global.

use std::io::{stderr, stdout, Write};

use clap::{command, Arg, ArgAction, ArgMatches, Command};
use json::object;

use crate::config::Config;
use crate::subcommands;

/// Returns the definitive clap::Command instance for git-global.
pub fn get_clap_app<'a>() -> Command<'a> {
    command!()
        .arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .action(ArgAction::SetTrue)
                .global(true)
                .help("Output subcommand results in JSON."),
        )
        .arg(
            Arg::new("untracked")
                .short('u')
                .long("untracked")
                .action(ArgAction::SetTrue)
                .conflicts_with("nountracked")
                .global(true)
                .help("Show untracked files in output."),
        )
        .arg(
            Arg::new("nountracked")
                .short('t')
                .long("nountracked")
                .action(ArgAction::SetTrue)
                .conflicts_with("untracked")
                .global(true)
                .help("Don't show untracked files in output."),
        )
        .subcommands(
            subcommands::get_subcommands()
                .iter()
                .map(|(cmd, desc)| Command::new(*cmd).about(*desc)),
        )
}

/// Merge command-line arguments from an ArgMatches object with a Config.
fn merge_args_with_config(config: &mut Config, matches: &ArgMatches) {
    if matches.get_flag("untracked") {
        config.show_untracked = true;
    }
    if matches.get_flag("nountracked") {
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
    let use_json = matches.get_flag("json");
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
