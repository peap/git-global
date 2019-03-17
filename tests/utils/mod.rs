extern crate git2;
extern crate git_global;
extern crate tempdir;

use std::path::{Path, PathBuf};

/// Initialize an empty git repo in a temporary directory, then run a closure
/// that takes that Repo instance.
#[allow(dead_code)]
pub fn with_temp_repo<T>(test: T) -> ()
where
    T: FnOnce(git_global::Repo) -> (),
{
    let tempdir = tempdir::TempDir::new("git-global-test").unwrap();
    let repo_path = tempdir.path();
    git2::Repository::init(repo_path).unwrap();
    let repo = git_global::Repo::new(repo_path.to_str().unwrap().to_string());
    test(repo);
}

/// Create a temporary directory with three empty git repos within, a, b, and c,
/// then run a closure that takes a reference to that base directory's Path.
#[allow(dead_code)]
pub fn with_base_dir_of_three_repos<T>(test: T) -> ()
where
    T: FnOnce(&Path) -> (),
{
    let tempdir = tempdir::TempDir::new("git-global-test").unwrap();
    let base_path = tempdir.path();
    for repo_name in ["a", "b", "c"].iter() {
        let mut repo_path = PathBuf::from(base_path);
        repo_path.push(repo_name);
        git2::Repository::init(repo_path).unwrap();
    }
    test(&base_path);
}
