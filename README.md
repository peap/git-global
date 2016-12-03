git-global
==========

Use `git-global` to keep track of all your local git repositories.

* `git global [list]`: listing of all the git repos on your machine
* `git global dirty`: all repos that have changes of any kind
* `git global unstaged`: all repos that have unstaged changes
* `git global staged`: all repos that have staged changes
* `git global stashed`: all repos that have stashed changes
* `git global branched`: all repos not on `master` (TODO: or a different
  default defined... somewhere?)
* `git global duplicates`: repos that are checked out to multiple places
* `git global remotes`: list all remotes (TODO: why? maybe filter by hostname?)
