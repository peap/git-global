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
fn test_scan() {
    utils::with_base_dir_of_three_repos(|config| {
        let report = scan::execute(config).unwrap();
        // There's one global message about how many repos were found.
        assert_eq!(report.messages.len(), 1);
        assert_eq!(
            report.messages[0],
            "Found 3 repos. Use `git global list` to show them."
        );
        // The per-repo message lists should be empty.
        for (_, msg_list) in &report.repo_messages {
            assert_eq!(msg_list.len(), 0);
        }
    });
}

#[test]
#[ignore]
fn test_status() {
    // to-do
}
