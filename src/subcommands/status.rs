//! The `status` subcommand, which shows `git status -s` for all known repos.

use std::io::{Write, stderr};

use git2;

use core::{GitGlobalResult, get_repos};
use errors::Result;

fn get_short_format_status(path: &str, status: git2::Status) -> String {
    // From git2's examples/status.rs...
    let mut istatus = match status {
        s if s.contains(git2::STATUS_INDEX_NEW) => 'A',
        s if s.contains(git2::STATUS_INDEX_MODIFIED) => 'M',
        s if s.contains(git2::STATUS_INDEX_DELETED) => 'D',
        s if s.contains(git2::STATUS_INDEX_RENAMED) => 'R',
        s if s.contains(git2::STATUS_INDEX_TYPECHANGE) => 'T',
        _ => ' ',
    };
    let mut wstatus = match status {
        s if s.contains(git2::STATUS_WT_NEW) => {
            if istatus == ' ' { istatus = '?'; } '?'
        }
        s if s.contains(git2::STATUS_WT_MODIFIED) => 'M',
        s if s.contains(git2::STATUS_WT_DELETED) => 'D',
        s if s.contains(git2::STATUS_WT_RENAMED) => 'R',
        s if s.contains(git2::STATUS_WT_TYPECHANGE) => 'T',
        _ => ' ',
    };
    if status.contains(git2::STATUS_IGNORED) {
        istatus = '!';
        wstatus = '!';
    }
    // TODO: handle submodule statuses?
    format!("{}{} {}", istatus, wstatus, path)
}

pub fn get_results() -> Result<GitGlobalResult> {
    let repos = get_repos();
    let mut result = GitGlobalResult::new(&repos);
    result.pad_repo_output();
    for repo in repos.iter() {
        let git2_repo = match repo.as_git2_repo() {
            None => {
                writeln!(&mut stderr(),
                    "Could not open {} as a git repo. Perhaps you should run \
                    `git global scan` again.", repo
                ).expect("failed to write to STDERR");
                continue;
            }
            Some(repo) => repo,
        };
        let mut opts = git2::StatusOptions::new();
        opts.show(git2::StatusShow::IndexAndWorkdir)
            .include_ignored(false);
        let statuses = git2_repo.statuses(Some(&mut opts))
            .expect(&format!("Could not get statuses for {}.", repo));
        for entry in statuses.iter() {
            let path = entry.path().unwrap();
            let status = entry.status();
            let status_for_path = get_short_format_status(path, status);
            result.add_repo_message(repo, format!("{}", status_for_path));
        }
    }
    Ok(result)
}
