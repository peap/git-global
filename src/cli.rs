//! The command line interface for git-global.

use std::io::{Write, stderr, stdout};

use clap::{Arg, ArgAction, ArgMatches, Command, command};
use serde_json::json;

use crate::config::Config;
use crate::subcommands;

/// Returns the definitive clap::Command instance for git-global.
pub fn get_clap_app() -> Command {
    command!()
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .global(true)
                .help("Enable verbose mode."),
        )
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
                .map(|(cmd, desc)| {
                    let mut subcmd = Command::new(*cmd).about(*desc);
                    if *cmd == "ignore" {
                        subcmd = subcmd.arg(
                            Arg::new("pattern")
                                .help("Pattern to add to global.ignore (matches paths containing this string)")
                                .required(true)
                                .index(1),
                        );
                    }
                    subcmd
                }),
        )
}

/// Merge command-line arguments from an ArgMatches object with a Config.
fn merge_args_with_config(config: &mut Config, matches: &ArgMatches) {
    if matches.get_flag("verbose") {
        config.verbose = true;
    }
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

    // Extract additional arguments for subcommands that need them
    let args = matches.subcommand().and_then(|(name, sub_matches)| {
        if name == "ignore" {
            sub_matches.get_one::<String>("pattern").map(|s| s.as_str())
        } else {
            None
        }
    });

    let report = subcommands::run(matches.subcommand_name(), config, args);
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
                let out = json!({
                    "error": true,
                    "message": format!("{}", err)
                });
                writeln!(&mut stderr(), "{:#}", out).unwrap();
            } else {
                writeln!(&mut stderr(), "{}", err).unwrap();
            }
            1
        }
    }
}
