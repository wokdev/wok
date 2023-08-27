# Philosophy behind Wok

## Wok is a kind of a package manager

Wok operates over several git repositories at once. The main approach is somewhat close to a package manager like `cargo`, `poetry`, or `npm`.

Thus, we are going to treat the `wok.yaml` config as an analog to a main package manager's config like `cargo` treats `Cargo.toml` and submodule objects committed to the main repo will be treated as an analog to a lock file.

## Principles

* Wok never commits anything (unless specifically asked to)
* Wok requires a git repo to operate and store `wok.yaml`
* Wok implements common package manager use cases
* Wok implements some typical multi-repo project use cases including development and release management flows
* Wok is not a replacement for the `git submodule` command despite having partially overlapped functionality
* Wok is not a replacement for the `git switch` command and avoids any overlap with it in the functionality

## Challenges

### Multiple repos

Wok commands operate over several isolated git repos. This could result in an operation's success on one repo and fail on another. Thus, any part of an operation should be able to either detect a possible error before the whole operation starts or roll back changes in case one of the next actions fails.

### Locking is commit

Locking dependencies added with Wok implies committing changes of submodule objects to the main repo. `git submodule` has some problem and just adds changes to the index. It looks like a reasonable alternative. Let's do that.

## Informing user on actions made

A single Wok command could perform a number of isolated actions. Each action's progress and the result should be communicated to the user. This should be addressed with the proper internal design.

## CLI Design

Wok's CLI could be separated into the following logical sections:
* housekeeping:
  - `init` -- create `wok.yaml`
  - `status` -- show the list of subprojects in wok config and for each repo: the current branch in the wok config, whether the checked out branch differed, whether it is clean
* package management:
  - `add` -- add a subproject to the config, add submodule, check out the repo, add changes to the index
  - `remove` -- remove a subproject from the config, remove submodule, remove the directory, add changes to the index
  - `update` -- switch each repo to its configured branch, fetch changes, update using the strategy configured in git, add changes to the index
  - `lock` -- ensure each repo is switched to its configured branch, do not fetch changes, add changes to the index
* development flow:
  - `switch [--create] [--all] [--branch <branch_name>] [[[<repo>] <repo>] …]` -- switch each `<repo>` to the current main repo branch in the config as in `git switch`, perform actions as in `wok lock` on those repos; with `--create` -- create the branch in repos if it doesn't exist; with `--all` -- act as all configured repos were provided; with `--branch <branch_name>` -- use `<branch_name>` instead of the current main repo branch
  - `push` [-u|--set-upstream] [--all] [--branch <branch_name>] [[[<repo>] <repo>] …] -- ensure  each `<repo>` is switched to its configured branch and then perform git push on those repos; if no `<repo>` provided act on all repos matching current main repo branch; with `--set-upstream`(`-u`) add upstream reference if not already configured; with `--all` -- act as all configured repos were provided; with `--branch <branch_name>` -- act only on repos currently configured to be on `<branch_name>`
* release flow:
  - `tag [--all] [--branch <branch_name>] [-s|--sign] [--push] [<tagname> [[[<repo>] <repo>] …]]` -- add a tag reference `<tagname>` to each `<repo>`; without `<tagname>` and `<repo>` show tag references closest to the current `HEAD` reference for each configured repo; with `--all` -- act as all configured repos were provided; with `--branch <branch_name>` -- act only on repos currently configured to be on `<branch_name>`; with `--sign`(`-s`) -- make a signed tag in a way similar to `git tag -s`; with `push` -- push created references immediately
