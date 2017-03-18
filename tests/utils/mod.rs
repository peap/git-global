extern crate git2;
extern crate tempdir;
extern crate git_global;

use std::path::PathBuf;

#[allow(dead_code)]
/// Initialize an empty git repo in a temporary directory, then run a closure
/// that takes that Repo instance.
pub fn with_temp_repo<T>(test: T) -> ()
    where T: FnOnce(git_global::Repo) -> () {

    let tempdir = tempdir::TempDir::new("git-global-test").unwrap();
    let repo_path = tempdir.path();
    git2::Repository::init(repo_path).unwrap();
    let repo = git_global::Repo::new(repo_path.to_path_buf());
    test(repo);
}

#[allow(dead_code)]
/// Create a temporary directory with three empty git repos within, a, b, and c,
/// then run a closure that takes a `Config` instance with that directory as its
/// scan root.
pub fn with_root_dir_of_three_repos<T>(test: T) -> ()
    where T: FnOnce(git_global::Config) -> () {

    let tempdir = tempdir::TempDir::new("git-global-test").unwrap();
    let scan_root = tempdir.path();
    for repo_name in ["a", "b", "c"].iter() {
        let mut repo_path = PathBuf::from(scan_root);
        repo_path.push(repo_name);
        git2::Repository::init(repo_path).unwrap();
    }
    let mut config = git_global::Config::new();
    config.scan_root = scan_root.to_path_buf();
    config.cache_filename = "test-scan.txt".to_string();
    test(config);
}
