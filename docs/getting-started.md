# Getting Started with Git Wok

## Introduction

**Git Wok** seamlessly manages multiple Git repositories simultaneously. Its core methodology bears a resemblance to popular package managers such as `cargo`, `poetry`, or `npm`.

In this context, we consider the `wok.yaml` configuration file as equivalent in importance to a package manager's primary configuration file, similar to how `Cargo.toml` is treated in `cargo`.

Submodule objects, once they've been committed to the primary repository, assume the role of a lock file counterpart.


## Installation

```sh
cargo install git-wok@0.3.0-dev
```

## Initializing Workspace

```sh
mkdir my-project-space && cd my-project-space
git init
git-wok init
```
