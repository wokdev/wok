# Getting Started with Git Wok

## Introduction

**Git Wok** seamlessly manages multiple Git repositories simultaneously. Its core methodology bears a resemblance to popular package managers such as `cargo`, `poetry`, or `npm`.

The `wok.toml` configuration file is the heart of your workspace, similar to how `Cargo.toml` is central to Rust projects or `package.json` to Node.js projects.

Submodule objects, once committed to the umbrella repository, serve as a lock file—capturing the exact state of each repository at a specific point in time.

## Installation

### From crates.io

```sh
cargo install git-wok
```

### From source

```sh
git clone https://codeberg.org/wok/wok
cd wok
cargo install --path .
```

## Creating a Workspace

Git Wok offers two approaches to create a workspace:

### Option 1: Initialize in an Existing Git Repository

Use this when you already have a git repository with submodules:

```sh
cd my-project-space
git-wok init
```

This will:
- Create a `wok.toml` configuration file
- Introspect existing submodules and add them to the config
- Optionally switch submodules to match your current branch

### Option 2: Assemble a New Workspace

Use this to convert a directory with multiple git repositories into a workspace:

```sh
# Create workspace directory with component repos
mkdir my-workspace
cd my-workspace

# Create or clone component repositories
git clone https://github.com/user/api.git
git clone https://github.com/user/frontend.git
git clone https://github.com/user/docs.git

# Assemble them into a workspace
git-wok assemble .
```

The `assemble` command will:
- Initialize the umbrella repository if needed
- Register each subdirectory as a git submodule
- Create the `wok.toml` configuration file
- Preserve remote URLs from component repositories

## Basic Workflow

### Check Status

View the status of all repositories in your workspace:

```sh
git-wok status
```

Example output:
```
On branch 'main', all clean
- 'api' is on branch 'main', all clean
- 'frontend' is on branch 'develop'
- 'docs' is on branch 'main', all clean
```

### Add a Repository

Add an existing submodule to your workspace configuration:

```sh
git submodule add https://github.com/user/new-component.git component
git-wok repo add component
```

### Switch Branches

Switch all repositories to match the umbrella's branch:

```sh
git checkout feature-x
git-wok head switch
```

Or switch with more control:

```sh
# Switch specific repos
git-wok switch api frontend

# Switch all repos
git-wok switch --all

# Switch to a different branch
git-wok switch --all --branch develop

# Create branch if it doesn't exist
git-wok switch --all --create --branch feature-y
```

### Update Repositories

Fetch and merge latest changes from remotes:

```sh
git-wok update
```

This will:
- Fetch changes from remote for each repository
- Merge changes into the configured branch
- Commit updated submodule state to umbrella repo

Use `--no-commit` to stage changes without committing:

```sh
git-wok update --no-commit
```

### Lock Submodule State

Commit the current submodule commit references:

```sh
git-wok lock
```

This ensures the umbrella repository captures exact commit hashes of all submodules.

### Push Changes

Push changes from all repositories:

```sh
# Push repos matching current branch
git-wok push

# Push all configured repos
git-wok push --all

# Set upstream for new branches
git-wok push --all -u
```

### Tag Releases

Create and manage tags across repositories:

```sh
# List tags in all repos on current branch
git-wok tag

# Create tag in all repos
git-wok tag --create v1.0.0 --all

# Create signed tag and push
git-wok tag --create v1.0.0 --all --sign --push

# Tag specific repos
git-wok tag --create v1.0.0 api frontend
```

## Advanced Features

### Selective Repository Targeting

Most commands support three targeting modes:

1. **Branch-based (default)**: Operate on repos matching the umbrella's current branch
2. **All repos**: Use `--all` flag to operate on all configured repos
3. **Specific repos**: List repo paths explicitly

Examples:
```sh
# Only repos on 'main' branch
git checkout main
git-wok push

# All configured repos
git-wok push --all

# Specific repos only
git-wok push api docs
```

### Skip Lists

Exclude repositories from bulk operations while retaining the ability to target them explicitly:

Edit `wok.toml`:
```toml
[[repo]]
path = "archived-component"
head = "main"
skip_for = ["push", "update", "tag"]
```

Now `git-wok push --all` will skip `archived-component`, but `git-wok push archived-component` will still work.

### Shell Completion

Generate shell completion scripts:

```sh
# Bash
git-wok completion bash > ~/.local/share/bash-completion/completions/git-wok

# Zsh
git-wok completion zsh > ~/.zsh/completions/_git-wok

# Fish
git-wok completion fish > ~/.config/fish/completions/git-wok.fish
```

## Configuration File

The workspace configuration lives in `wok.toml` at the root of your umbrella repository. See the [Wokfile documentation](wokfile.md) for detailed syntax.

## Common Workflows

### Feature Development Workflow

```sh
# Start feature branch
git checkout -b feature-new-api
git-wok switch --all --create --branch feature-new-api

# Make changes in subrepositories...
# (work in api/, frontend/, etc.)

# Check status
git-wok status

# Lock state periodically
git-wok lock

# Push changes
git-wok push --all -u

# Merge and tag release
git checkout main
git merge feature-new-api
git-wok tag --create v2.0.0 --all --sign --push
```

### Maintenance Workflow

```sh
# Update all repos to latest
git-wok update

# Review changes
git-wok status

# If satisfied, the umbrella repo now has a commit with updated submodules
git push
```

### Adding a New Repository

```sh
# Add as submodule first
git submodule add https://github.com/user/new-repo.git new-repo

# Register with wok
git-wok repo add new-repo

# Initialize and switch to current branch
git-wok switch --create new-repo

# Commit configuration
git add wok.toml .gitmodules new-repo
git commit -m "Add new-repo to workspace"
```

## Troubleshooting

### "Git Wok file not found"

Ensure you're in the umbrella repository root, or use `-f` to specify the path:

```sh
git-wok -f path/to/wok.toml status
```

### "Branch does not exist"

Use `--create` when switching to a new branch:

```sh
git-wok switch --all --create --branch new-feature
```

### Merge Conflicts During Update

The `update` command will report conflicts. Resolve them manually in each affected repository, then commit:

```sh
cd affected-repo
# Resolve conflicts
git add .
git commit

cd ..
git-wok lock  # Update umbrella with resolved state
```

### Submodule Not Initialized

Initialize submodules if needed:

```sh
git submodule update --init --recursive
```

## Next Steps

- Read the [CLI Reference](cli.md) for detailed command documentation
- Learn about [Wokfile syntax](wokfile.md) for advanced configuration
- Check the [Roadmap](ROADMAP.md) for upcoming features
