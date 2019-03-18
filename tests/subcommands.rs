extern crate git_global;

mod utils;

use git_global::subcommands;

#[test]
fn test_info() {
    utils::with_base_dir_of_three_repos(|config| {
        let report = subcommands::info::execute(config).unwrap();
        // There's a series of global messages about git-global itself.
        assert_eq!(report.messages.len(), 7);
        // The per-repo message lists should be empty.
        for (_, msg_list) in &report.repo_messages {
            assert_eq!(msg_list.len(), 0);
        }
    });
}

#[test]
fn test_list() {
    utils::with_base_dir_of_three_repos(|config| {
        let report = subcommands::list::execute(config).unwrap();
        // There are no global messages.
        assert_eq!(report.messages.len(), 0);
        // The per-repo message lists should have one empty string each.
        for (_, msg_list) in &report.repo_messages {
            assert_eq!(msg_list.len(), 1);
            assert_eq!(msg_list[0], "");
        }
    });
}

#[test]
fn test_scan() {
    utils::with_base_dir_of_three_repos(|config| {
        let report = subcommands::scan::execute(config).unwrap();
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
fn test_status() {
    utils::with_base_dir_of_three_repos(|config| {
        let report = subcommands::status::execute(config).unwrap();
        // There are no global messages.
        assert_eq!(report.messages.len(), 0);
        // There are no per-repo messages, because all the repos are clean.
        for (_, msg_list) in &report.repo_messages {
            assert_eq!(msg_list.len(), 0);
        }
    });
}
