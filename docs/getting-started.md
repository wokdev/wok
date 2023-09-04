# Getting Started with Wok

Wok is a tool to manage multiple git repositories simultaneously.

In a nut shell, Wok is a package manager for git submodules. You choose a repo which stores Wok Workspace configuration, add subrepos, and Wok will help you manage them in a way you expect from a package manager to do it.

## Installation

```sh
cargo install wok-dev@0.3.0-dev
```

## Initializing Workspace

```sh
mkdir my-project-space && cd my-project-space
git init
wok init
```
