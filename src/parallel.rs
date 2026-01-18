//! Thread-limited parallel execution for git-global subcommands.
//!
//! Provides a helper to run operations across multiple repos with a bounded
//! number of concurrent threads, avoiding resource exhaustion.

use std::sync::mpsc;
use std::thread;

use crate::repo::Repo;

/// Returns the default number of threads to use for parallel operations.
pub fn default_parallelism() -> usize {
    num_cpus::get()
}

/// Executes a function across multiple repos with limited concurrency.
///
/// Uses a semaphore pattern with `sync_channel` to limit the number of
/// concurrent threads, preventing resource exhaustion when processing
/// many repositories.
///
/// # Arguments
/// * `repos` - The repositories to process
/// * `max_threads` - Maximum number of concurrent threads
/// * `f` - Function to execute for each repo, receives the Repo and returns T
///
/// # Returns
/// A vector of (repo_path, result) tuples in completion order
pub fn run_parallel<T, F>(repos: Vec<Repo>, max_threads: usize, f: F) -> Vec<(String, T)>
where
    T: Send + 'static,
    F: Fn(&Repo) -> T + Send + Sync + Clone + 'static,
{
    let n_repos = repos.len();
    if n_repos == 0 {
        return vec![];
    }

    // Channel for results
    let (result_tx, result_rx) = mpsc::channel();

    // Semaphore channel to limit concurrency - blocks when full
    let (permit_tx, permit_rx) = mpsc::sync_channel::<()>(max_threads);

    // Pre-fill permits
    for _ in 0..max_threads {
        permit_tx.send(()).unwrap();
    }

    // Spawn threads, gated by permits
    for repo in repos {
        // Wait for a permit (blocks if max_threads are already running)
        permit_rx.recv().unwrap();

        let result_tx = result_tx.clone();
        let permit_tx = permit_tx.clone();
        let f = f.clone();

        thread::spawn(move || {
            let path = repo.path();
            let result = f(&repo);
            result_tx.send((path, result)).unwrap();
            // Return the permit so another thread can start
            let _ = permit_tx.send(());
        });
    }

    // Collect all results
    let mut results = Vec::with_capacity(n_repos);
    for _ in 0..n_repos {
        results.push(result_rx.recv().unwrap());
    }

    results
}
