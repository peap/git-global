mod utils;

use std::env;
use std::io::Cursor;
use std::path::PathBuf;

use clap::crate_version;
use regex::{Regex, escape};

use git_global::test_utils::TestEnv;
use git_global::{Report, subcommands};

fn report_to_string(report: &Report) -> String {
    let mut out = Cursor::new(Vec::new());
    report.print(&mut out);
    String::from_utf8(out.into_inner()).unwrap()
}

#[test]
fn test_info() {
    utils::with_base_dir_of_three_repos(|mut config| {
        let basedir = config.basedir.clone();
        let cache = config
            .cache_file
            .clone()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        if config.manpage_file.is_none() {
            config.manpage_file = Some(PathBuf::from("/test"));
        }
        let manpage = config
            .manpage_file
            .clone()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let report = subcommands::info::execute(config).unwrap();
        let expected = vec![
            format!(r"^git-global {}$", crate_version!()),
            r"^============+".to_string(),
            r"^Number of repos: 3$".to_string(),
            format!(r"^Base directory: {}$", escape(basedir.to_str().unwrap())),
            r"^Ignored patterns:$".to_string(),
            r"^Default command: status$".to_string(),
            r"^Verbose: false$".to_string(),
            r"^Show untracked: true$".to_string(),
            format!(r"^Cache file: {}$", escape(&cache)),
            r"^Cache file age: 0d, 0h, 0m, .s$".to_string(),
            format!(r"^Manpage file: {}$", escape(&manpage)),
            format!(r"^Detected OS: {}$", escape(env::consts::OS)),
            r"^$".to_string(),
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
        let expected = [
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
        let report = subcommands::scan::execute(config, vec![]).unwrap();
        // There is one global message about the three repos we found.
        assert_eq!(
            report_to_string(&report),
            "Found 3 repos. Use `git global list` to show them.\n"
        );
    });
}

#[test]
fn test_staged() {
    utils::with_base_dir_of_three_repos(|config| {
        let report = subcommands::staged::execute(config).unwrap();
        // There are no global messages.
        assert_eq!(report_to_string(&report), "");
    });
}

#[test]
fn test_stashes() {
    utils::with_base_dir_of_three_repos(|config| {
        let report = subcommands::stashed::execute(config).unwrap();
        // There are no global messages.
        assert_eq!(report_to_string(&report), "");
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

#[test]
fn test_unstaged() {
    utils::with_base_dir_of_three_repos(|config| {
        let report = subcommands::unstaged::execute(config).unwrap();
        // There are no global messages.
        assert_eq!(report_to_string(&report), "");
    });
}

#[test]
fn test_ahead() {
    let mut env = TestEnv::new();
    env.create_repo("repo-ahead")
        .commit("file.txt", "initial")
        .setup_remote()
        .commit("file.txt", "ahead")
        .build();

    let config = env.config();
    let repo_path = env.tempdir.path().join("repo-ahead");

    let report = subcommands::ahead::execute(config).unwrap();
    let output = report_to_string(&report);
    assert!(output.contains(repo_path.to_str().unwrap()));
}

#[test]
fn test_staged_with_changes() {
    let mut env = TestEnv::new();
    env.create_repo("repo-staged")
        .stage("staged.txt", "content")
        .build();

    let config = env.config();
    let repo_path = env.tempdir.path().join("repo-staged");

    let report = subcommands::staged::execute(config).unwrap();
    let output = report_to_string(&report);
    assert!(output.contains(repo_path.to_str().unwrap()));
}

#[test]
fn test_ignore_and_ignored() {
    let env = TestEnv::new();
    let gitconfig_path = env.write_gitconfig();

    let config = git_global::Config::from_gitconfig(&gitconfig_path);

    // Verify isolation: basedir should come from the gitconfig we wrote.
    assert_eq!(config.basedir, env.tempdir.path());

    subcommands::ignore::execute(config, "my-pattern").unwrap();

    let config = git_global::Config::from_gitconfig(&gitconfig_path);
    let report = subcommands::ignored::execute(config).unwrap();
    let output = report_to_string(&report);
    assert!(output.contains("my-pattern"));
}

#[test]
fn test_install_manpage_with_dest() {
    let env = TestEnv::new();
    let dest_path = env.tempdir.path().join("manpage.1");
    let config = env.config();

    let report =
        subcommands::install_manpage::execute(config, Some(dest_path.clone()))
            .unwrap();
    let output = report_to_string(&report);
    assert!(output.contains("Installed manpage to"));
    assert!(dest_path.exists());
    let content = std::fs::read_to_string(&dest_path).unwrap();
    assert!(content.contains(".TH git-global 1"));
}

#[test]
fn test_auto_install_manpage_setting() {
    let env = TestEnv::new();
    let mut config = env.config();
    let dest_path = env.tempdir.path().join("auto_manpage.1");

    // Disabled setting: should not write
    config.manpage_file = Some(dest_path.clone());
    config.auto_install_manpage = false;
    config.maybe_auto_install_manpage();
    assert!(!dest_path.exists());

    // Enabled setting: should write
    config.auto_install_manpage = true;
    config.maybe_auto_install_manpage();
    assert!(dest_path.exists());
}

#[test]
fn test_auto_install_manpage_upgrades() {
    let env = TestEnv::new();
    let mut config = env.config();
    let dest_path = env.tempdir.path().join("upgrade_manpage.1");

    // Write an older version of the manpage
    std::fs::write(&dest_path, "old manpage version 0.1.0").unwrap();

    config.manpage_file = Some(dest_path.clone());
    config.auto_install_manpage = true;
    config.maybe_auto_install_manpage();

    // Verify it was overwritten with current version
    let updated = std::fs::read_to_string(&dest_path).unwrap();
    assert!(updated.contains(clap::crate_version!()));
    assert!(updated.contains(".TH git-global 1"));
}
