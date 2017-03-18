extern crate git_global;

mod utils;

use git_global::subcommands;

#[test]
#[ignore]
fn test_help() {
    // TODO
}

#[test]
#[ignore]
fn test_info() {
    // TODO
}

#[test]
#[ignore]
fn test_list() {
    // TODO
}

#[test]
fn test_scan() {
    utils::with_root_dir_of_three_repos(|config| {
        let report_result = subcommands::scan::run(&config);
        assert!(report_result.is_ok());
        let report = report_result.unwrap();
        assert_eq!(3, report.get_repos().len());
    });
}

#[test]
#[ignore]
fn test_status() {
    // TODO
}

