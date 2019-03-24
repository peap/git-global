mod utils;

#[test]
/// Test that we get an actual git repo, we can get a git2::Repository
/// reference to it, and it's not bare.
fn test_repo_initialization() {
    utils::with_temp_repo(|repo| {
        let git2_repo = repo.as_git2_repo();
        assert!(!git2_repo.is_bare());
    });
}
