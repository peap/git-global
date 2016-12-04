git-global
==========

Use `git-global` to keep track of all your local git repositories. Put it
on your `PATH` with `cargo install git-global`, and you gain an extra
git subcommand that you can run from anywhere.

Use `git global <subcommand>` to:

* `git global [list]`: show all the git repos on your machine (the default
  subcommand)
* `git global status`: show `git status` for all your git repos
* `git global unstaged`: show all repos that have unstaged changes
* `git global staged`: show all repos that have staged changes
* `git global stashed`: show all repos that have stashed changes
* `git global dirty`: show all repos that have changes of any kind
* `git global branched`: show all repos not on `master` (TODO: or a different
  default branch that is defined... somewhere?)
* `git global duplicates`: show repos that are checked out to multiple places
* `git global remotes`: show all remotes (TODO: why? maybe filter by hostname?)

Ideas
-----

* `git global scan`: search to git repos
* `git global update`: update git repos list
* `git global monitor`: launch a daemon to watch git dirs with inotify
