use crate::Config;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestEnv {
    pub tempdir: TempDir,
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl TestEnv {
    pub fn new() -> Self {
        let tempdir = TempDir::new().unwrap();
        TestEnv {
            tempdir,
        }
    }

    pub fn create_repo(&mut self, name: &str) -> RepoBuilder<'_> {
        let repo_path = self.tempdir.path().join(name);
        create_dir_all(&repo_path).unwrap();
        let mut opts = git2::RepositoryInitOptions::new();
        opts.initial_head("master");
        let repo = git2::Repository::init_opts(&repo_path, &opts).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
        RepoBuilder {
            env: self,
            repo_path,
        }
    }

    pub fn write_gitconfig(&self) -> PathBuf {
        let gitconfig_path = self.tempdir.path().join(".gitconfig");
        let mut f = File::create(&gitconfig_path).unwrap();
        writeln!(f, "[global]\n\tbasedir = {}", self.tempdir.path().display())
            .unwrap();
        gitconfig_path
    }

    pub fn config(&self) -> Config {
        let gitconfig_path = self.write_gitconfig();
        let mut config = Config::from_gitconfig(&gitconfig_path);
        config.cache_file = Some(self.tempdir.path().join("repos.txt"));
        config.manpage_file = None;
        config
    }
}

pub struct RepoBuilder<'a> {
    env: &'a mut TestEnv,
    repo_path: PathBuf,
}

impl<'a> RepoBuilder<'a> {
    pub fn commit(self, filename: &str, content: &str) -> Self {
        let file_path = self.repo_path.join(filename);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let repo = git2::Repository::open(&self.repo_path).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new(filename)).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = repo.signature().unwrap();
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let mut parents = Vec::new();
        if let Some(ref p) = parent {
            parents.push(p);
        }
        repo.commit(Some("HEAD"), &sig, &sig, "commit", &tree, &parents)
            .unwrap();
        self
    }

    pub fn stage(self, filename: &str, content: &str) -> Self {
        let file_path = self.repo_path.join(filename);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let repo = git2::Repository::open(&self.repo_path).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new(filename)).unwrap();
        index.write().unwrap();
        self
    }

    pub fn unstaged(self, filename: &str, content: &str) -> Self {
        let file_path = self.repo_path.join(filename);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        self
    }

    pub fn stash(self, message: &str) -> Self {
        let mut repo = git2::Repository::open(&self.repo_path).unwrap();
        let sig = repo.signature().unwrap();
        repo.stash_save(&sig, message, None).unwrap();
        self
    }

    pub fn setup_remote(self) -> Self {
        let repo = git2::Repository::open(&self.repo_path).unwrap();
        let remote_path = self.env.tempdir.path().join(format!(
            "{}.git",
            self.repo_path.file_name().unwrap().to_str().unwrap()
        ));
        git2::Repository::init_bare(&remote_path).unwrap();
        repo.remote("origin", remote_path.to_str().unwrap())
            .unwrap();
        let mut remote = repo.find_remote("origin").unwrap();
        remote
            .push(&["refs/heads/master:refs/heads/master"], None)
            .unwrap();
        self
    }

    pub fn build(self) -> &'a mut TestEnv {
        self.env
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_env_and_repo_builder() {
        let mut env = TestEnv::new();
        env.create_repo("repo1")
            .commit("file1.txt", "content")
            .setup_remote()
            .commit("file1.txt", "new content")
            .build()
            .create_repo("repo2")
            .stage("dirty.txt", "stuff")
            .build()
            .create_repo("repo3")
            .unstaged("untracked.txt", "more stuff")
            .build();

        let config = env.config();
        assert_eq!(config.basedir, env.tempdir.path());
        // Verify that we can find the repos
        let mut config = config;
        let repos = config.get_repos();
        assert_eq!(repos.len(), 3);
    }
}
