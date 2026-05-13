use git_global::test_utils::TestEnv;
use git_global::{Config, Repo};

/// Initialize an empty git repo in a temporary directory, then run a closure
/// that takes that Repo instance.
#[allow(dead_code)]
pub fn with_temp_repo<T>(test: T)
where
    T: FnOnce(Repo),
{
    let mut env = TestEnv::new();
    env.create_repo("temp-repo").build();
    let repo_path = env.tempdir.path().join("temp-repo");
    let repo = Repo::new(repo_path);
    test(repo);
}

/// Create a temporary directory with three empty git repos within, a, b, and c,
/// then run a closure that takes a Config initialized for that temporary
/// directory.
#[allow(dead_code)]
pub fn with_base_dir_of_three_repos<T>(test: T)
where
    T: FnOnce(Config),
{
    let mut env = TestEnv::new();
    env.create_repo("a").build();
    env.create_repo("b").build();
    env.create_repo("c").build();
    let config = env.config();
    test(config);
}
