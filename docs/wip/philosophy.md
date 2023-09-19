# Philosophy behind Wok

!!! warning "Work in progress"

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
