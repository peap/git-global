#[macro_use]
extern crate clap;
extern crate git_global;
extern crate regex;

mod utils;

use std::io::Cursor;
use std::path::PathBuf;

use regex::{escape, Regex};

use git_global::{subcommands, Report};

fn report_to_string(report: &Report) -> String {
    let mut out = Cursor::new(Vec::new());
    report.print(&mut out);
    String::from_utf8(out.into_inner()).unwrap()
}

#[test]
fn test_info() {
    utils::with_base_dir_of_three_repos(|config| {
        let basedir = config.basedir.clone();
        let cache = config.cache_file.clone().to_str().unwrap().to_string();
        let report = subcommands::info::execute(config).unwrap();
        let expected = vec![
            format!(r"^git-global {}$", crate_version!()),
            format!(r"^============+"),
            format!(r"^Number of repos: 3$"),
            format!(r"^Base directory: {}$", escape(basedir.to_str().unwrap())),
            format!(r"^Cache file: {}$", escape(&cache)),
            format!(r"^Cache file age: 0d, 0h, 0m, .s$"),
            format!(r"^Ignored patterns:$"),
            format!(r"^Default command: status$"),
            format!(r"^Show untracked: true$"),
            format!(r"^$"),
        ];
        let output = report_to_string(&report);
        for (i, line) in output.lines().enumerate() {
            let pattern = &expected[i];
            let re = Regex::new(pattern).unwrap();
            assert!(
                re.is_match(line),
                "Line {} didn't match; got {}, want {}",
                i + 1,
                line,
                pattern
            )
        }
    });
}

#[test]
fn test_list() {
    utils::with_base_dir_of_three_repos(|config| {
        let basedir = config.basedir.clone();
        let report = subcommands::list::execute(config).unwrap();
        // There are no global messages; the per-repo messages are simply a list
        // of the repo paths themselves.
        let expected = vec![
            PathBuf::from(&basedir).join("a"),
            PathBuf::from(&basedir).join("b"),
            PathBuf::from(&basedir).join("c"),
        ];
        let output = report_to_string(&report);
        for (i, line) in output.lines().enumerate() {
            assert_eq!(expected[i].to_str().unwrap(), line);
        }
    });
}

#[test]
fn test_scan() {
    utils::with_base_dir_of_three_repos(|config| {
        let report = subcommands::scan::execute(config).unwrap();
        // There is one global message about the three repos we found.
        assert_eq!(
            report_to_string(&report),
            "Found 3 repos. Use `git global list` to show them.\n"
        );
    });
}

#[test]
fn test_status() {
    utils::with_base_dir_of_three_repos(|config| {
        let report = subcommands::status::execute(config).unwrap();
        // There are no global messages.
        assert_eq!(report_to_string(&report), "");
    });
}
