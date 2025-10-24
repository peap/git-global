# git-global

[![Crates.io](https://img.shields.io/crates/v/git-global.svg)](https://crates.io/crates/git-global)
[![Crates.io](https://img.shields.io/crates/d/git-global.svg)](https://crates.io/crates/git-global)
[![Build](https://github.com/peap/git-global/actions/workflows/rust.yml/badge.svg)](https://github.com/peap/git-global/actions)

Use `git-global` to keep track of all the git repositories on your machine.

This is a Rust program that you can install with `cargo install git-global`.
(To obtain `cargo` and Rust, see https://rustup.rs.) Once installed, you can
optionally install the manpage with `git global install-manpage`

Once installed, you gain an extra git subcommand that you can run from anywhere
to check up on all your git repos: `git global`.  Use `git global <subcommand>`
to:

* `git global ahead`: show repos where branches contain commits that are not
  present on any of the remotes
* `git global info`: show meta-information about git-global itself
  (configuration, number of known repos, etc.)
* `git global install-manpage`: (non-functional) attempt to install
  git-global's manpage
* `git global list`: show list of all known repos
* `git global scan`: update the cache of known repos by searching your
  filesystem
* `git global staged`: show status of the git index for repos with such changes
* `git global stashed`: show stashes for all repos that have them
* `git global status`: show `git status -s` for all your repos with any changes
* `git global unstaged`: show status of the working directory for repos with
  such changes

## Command-line flags

In addition to config-file-based options, there are a set of global
command-line flags that take precedence:

* `--json`: Print subcommand results in a JSON format.
* `--untracked`: Show untracked files in subcommand results, e.g., for the
  `status`, `staged`, and `unstaged` subcommands.
* `--nountracked`: Don't show untracked files in subcommand results, e.g., for
  the `status`, `staged`, and `unstaged` subcommands.

## Configuration

To change the default behavior of `git-global`, you can do so with --- wait for
it --- [git's global
configuration](https://git-scm.com/book/en/v2/Customizing-Git-Git-Configuration)!

To set the root directory for repo discovery to something other than your home
directory:
```
git config --global global.basedir /some/path
```

To add patterns to exclude while walking directories:
```
git config --global global.ignore .cargo,.vim,Library
```

The full list of configuration options supported in the `global` section of
`.gitconfig` is:

* `basedir`: The root directory for repo discovery (default: `$HOME`)
* `follow-symlinks`: Whether to follow symbolic links during repo discovery
  (default: `true`)
* `same-filesystem`: Whether to stay on the same filesystem as `basedir`
  during repo discovery
  ([on Unix or Windows only](https://docs.rs/walkdir/2.2.8/walkdir/struct.WalkDir.html#method.same_file_system))
  (default: `true` on Windows or Unix, `false` otherwise)
* `ignore`: Comma-separated list of patterns to exclude while walking
  directories (default: none)
* `default-cmd`: The default subcommand to run if unspecified, i.e., when
  running `git global` (default: `status`)
* `show-untracked`: Whether to include untracked files in output (default:
  `true`)

## Manpage generation

An up-to-date copy of the manpage lives in the repository at
[doc/git-global.1](doc/git-global.1). To generate it from a local clone of the
repo, run:

```
cargo run --bin generate-manpage --features=manpage > doc/git-global.1
```

## Ideas

The following are some ideas about future subcommands and features:

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

* `git global cd <fuzzy repo>`: change to the directory of the matched repo (#6)

* stream results to `STDOUT` as the come in (from `git global status`, for
  example, so we don't have to wait until they're all collected)
* use `locate .git` if the DB is populated, instead of walking the filesystem
* make a `Subcommand` trait
* do concurrency generically, not just for the `status` subcommand

## Release Notes

* 0.7.0 (2025-10-24)
  * Update to Rust 2024 edition.
  * Various dependency updates.
* 0.6.7 (2025-08-01)
  * Various dependency updates.
* 0.6.6 (2025-02-07)
  * Fix an alignment issue with the `--verbose` flag's output.
* 0.6.5 (2025-02-07)
  * Add a `-v`/`--verbose` flag, so far just used to indicate progress during
    `scan`'s directory walking. Useful for identifying patterns that should be
    omitted from scans.
* 0.6.4 (2025-01-01)
  * Various dependency updates.
* 0.6.3 (2024-08-10)
  * Make the `ahead` subcommand work with corrupted references (#105). Thanks,
    koalp!
  * Various dependency updates.
* 0.6.2 (2024-06-08)
  * Various dependency updates, including `json` --> `serde_json`.
* 0.6.1 (2023-08-10)
  * Various dependency updates.
* 0.6.0 (2023-05-10)
  * Update to Rust 2021 edition.
  * Update, replace, or remove several dependencies.
* 0.5.1 (2022-03-17)
  * Add the `generate-manpage` binary and (non-functional) `install-manpage`
    subcommand.
* 0.5.0 (2021-07-12)
  * Add the `ahead` subcommand - thanks, koalp!.
* 0.4.1 (2021-06-03)
  * Fix crashes when a cached repo has been deleted.
* 0.4.0 (2021-04-19)
  * Update to Rust 2018 edition (Thanks, koalp!).
  * Replace the `dirs` and `app_dirs` crates with `directories`.
    * Previously created cache files may be ignored after upgrading to this
      version, so the cache might need to regenerated during the first command
      run after upgrading to this version. However, we no longer panic if the
      cache file can't be created.
* 0.3.2 (2020-11-13)
  * Update dependencies.
* 0.3.1 (2020-04-25)
  * Update dependencies.
* 0.3.0 (2019-08-04)
  * Add subcommands:
    * `staged`
    * `stashed`
    * `unstaged`
  * Add config options:
    * `default-cmd`
    * `show-untracked`
    * `follow-symlinks`
    * `same-filesystem`
  * Add command-line flags:
    * `--untracked`
    * `--nountracked`
  * Add options to follow symlinks and stay on the same filesystem while
    scanning directories; both are `true` by default. (Thanks, pka!)
* 0.2.0 (2019-03-18)
  * Include untracked files in status output.
  * Expand documentation and package metadata.
  * Update and change several dependencies.
  * Add some tests.
  * Several public API changes, such as:
    * Rename `GitGlobalConfig` to `Config`.
    * Rename `GitGlobalResult` to `Report`.
    * Move `get_repos` `find_repos`, and `cache_repos` functions to `Config`.
    * Split the `core` module into `config`, `repo`, and `report`.
  * Merge bug fix for scanning directories when nothing is configured to be
    ignored ([#1](https://github.com/peap/git-global/pull/1)).
* 0.1.0 (2017-01-31)
  * Initial release with these subcommands: help, info, list, scan, status.
