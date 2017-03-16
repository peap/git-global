# git-global

[![Crates.io](https://img.shields.io/crates/v/git-global.svg)](https://crates.io/crates/git-global)
[![Crates.io](https://img.shields.io/crates/d/git-global.svg)](https://crates.io/crates/git-global)

Use `git-global` to keep track of all your git repositories.

This is a Rust program that you can put on your `PATH` with `cargo install
git-global`, gaining an extra git subcommand that you can run from anywhere. To
obtain cargo and Rust, see https://rustup.rs.

Use `git global <subcommand>` to:

* `git global [status]`: show `git status` for all your git repos (the default
  subcommand)
* `git global info`: show information about git-global itself (configuration,
  number of known repos, etc.)
* `git global list`: show all git repos git-global knows about
* `git global scan`: search your filesystem for git repos and update cache

## Configuration

To change the behavior of `git-global`, you can do so with --- wait for it
--- git global configuration!

To set the base path for search to something other than your home directory:
```
git config --global global.basedir /some/path
```

To add patterns to exclude while walking directories:
```
git config --global global.ignore .cargo,.vim,Library
```

## Ideas

* `git global unstaged`: show all repos that have unstaged changes
* `git global staged`: show all repos that have staged changes
* `git global stashed`: show all repos that have stashed changes
* `git global dirty`: show all repos that have changes of any kind
* `git global branched`: show all repos not on `master` (TODO: or a different
  default branch in .gitconfig)
* `git global duplicates`: show repos that are checked out to multiple places
* `git global remotes`: show all remotes (TODO: why? maybe filter by hostname?)

* `git global add <path>`: add a git repo to the cache that would not be found in a scan
* `git global ignore <path>`: ignore a git repo and remove it from the cache
* `git global ignored`: show which git repos are currently being ignored
* `git global monitor`: launch a daemon to watch git dirs with inotify
* `git global pull`: pull down changes from default tracking branch for clean repos

* stream results as the come in (from `git global status`, for example, so we don't
  have to wait until they're all collected)
* use `locate .git` if the DB is populated, instead of walking everything
* make a `Subcommand` trait
* do concurrency generically, not just for status subcommand
* rename `GitGlobalResult` so it's not confused with a normal `Result`

## Release Notes

* 0.1.1 (work-in-progress)
  * add tests
  * expand documentation and package metadata
  * update dependency versions
* 0.1.0 (1/31/17)
  * initial release with the following subcommands: help, info, list, scan, status
