extern crate git2;
extern crate git_global;
extern crate tempdir;

use std::path::PathBuf;

use self::git_global::{GitGlobalConfig, Repo};

/// Initialize an empty git repo in a temporary directory, then run a closure
/// that takes that Repo instance.
#[allow(dead_code)]
pub fn with_temp_repo<T>(test: T) -> ()
where
    T: FnOnce(Repo) -> (),
{
    let tempdir = tempdir::TempDir::new("git-global-test").unwrap();
    let repo_path = tempdir.path();
    git2::Repository::init(repo_path).unwrap();
    let repo = Repo::new(repo_path.to_str().unwrap().to_string());
    test(repo);
}

/// Create a temporary directory with three empty git repos within, a, b, and c,
/// then run a closure that takes a GitGlobalConfig initialized for that
/// temporary directory.
#[allow(dead_code)]
pub fn with_base_dir_of_three_repos<T>(test: T) -> ()
where
    T: FnOnce(GitGlobalConfig) -> (),
{
    let tempdir = tempdir::TempDir::new("git-global-test").unwrap();
    let base_path = tempdir.path();
    for repo_name in ["a", "b", "c"].iter() {
        let mut repo_path = PathBuf::from(base_path);
        repo_path.push(repo_name);
        git2::Repository::init(repo_path).unwrap();
    }
    let config = GitGlobalConfig{
        basedir: base_path.clone().to_str().unwrap().to_string(),
        ignored_patterns: vec![],
        cache_file: base_path.clone().join("test-cache-file.txt").to_path_buf(),
    };
    test(config);
}
