extern crate git_global;

mod utils;

use git_global::subcommands::scan;

#[test]
#[ignore]
fn test_help() {
    // to-do
}

#[test]
#[ignore]
fn test_info() {
    // to-do
}

#[test]
#[ignore]
fn test_list() {
    // to-do
}

#[test]
#[ignore]
fn test_scan() {
    utils::with_base_dir_of_three_repos(|ref _path| {
        // TODO: inject a GitGlobalConfig that takes `path` as its base directory
        let result = scan::get_results();
        assert!(result.is_ok());
    });
}

#[test]
#[ignore]
fn test_status() {
    // to-do
}
