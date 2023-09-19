# Planned Wok CLI

!!! warning "Work in progress"

## CLI Design

Wok's CLI could be separated into the following logical sections:


### Housekeeping

- `init` -- create `wok.yaml`
- `status` -- show the list of subprojects in wok config and for each repo: the current branch in the wok config, whether the checked out branch differed, whether it is clean


### Package Management

- `add` -- add a subproject to the config, add submodule, check out the repo, add changes to the index
- `remove` -- remove a subproject from the config, remove submodule, remove the directory, add changes to the index
- `update` -- switch each repo to its configured branch, fetch changes, update using the strategy configured in git, add changes to the index
- `lock` -- ensure each repo is switched to its configured branch, do not fetch changes, add changes to the index


### Development Flow

- `switch [--create] [--all] [--branch <branch_name>] [[[<repo>] <repo>] …]` -- switch each `<repo>` to the current main repo branch in the config as in `git switch`, perform actions as in `wok lock` on those repos;
  - with `--create` -- create the branch in repos if it doesn't exist;
  - with `--all` -- act as all configured repos were provided;
  - with `--branch <branch_name>` -- use `<branch_name>` instead of the current main repo branch
- `push` [-u|--set-upstream] [--all] [--branch <branch_name>] [[[<repo>] <repo>] …] -- ensure  each `<repo>` is switched to its configured branch and then perform git push on those repos;
  - if no `<repo>` provided act on all repos matching current main repo branch;
  - with `--set-upstream`(`-u`) add upstream reference if not already configured;
  - with `--all` -- act as all configured repos were provided;
  - with `--branch <branch_name>` -- act only on repos currently configured to be on `<branch_name>`


### Release Flow

- `tag [--all] [--branch <branch_name>] [-s|--sign] [--push] [<tagname> [[[<repo>] <repo>] …]]` -- add a tag reference `<tagname>` to each `<repo>`;
  - without `<tagname>` and `<repo>` show tag references closest to the current `HEAD` reference for each configured repo;
  - with `--all` -- act as all configured repos were provided;
  - with `--branch <branch_name>` -- act only on repos currently configured to be on `<branch_name>`;
  - with `--sign`(`-s`) -- make a signed tag in a way similar to `git tag -s`;
  - with `push` -- push created references immediately
