# Getting Started with Git Wok

## Introduction

**Git Wok** seamlessly manages multiple Git repositories simultaneously. Its core methodology resembles popular package managers such as `cargo`, `poetry`, or `npm`.

The `wok.toml` configuration file is the heart of your workspace, similar to how `Cargo.toml` is central to Rust projects or `package.json` to Node.js projects.

Submodule objects, once committed to the umbrella repository, serve as a lock file—capturing the exact state of each repository at a specific commit.

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
wok init
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
wok assemble .
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
wok status
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
wok add component
```

### Switch Branches

Switch all repositories to match the umbrella's branch:

```sh
git checkout feature-x
wok switch --all
```

Or switch with more control:

```sh
# Switch specific repos
wok switch api frontend

# Switch all repos
wok switch --all

# Switch to a different branch
wok switch --all --branch develop

# Create branch if it doesn't exist
wok switch --all --create --branch feature-y
```

### Update Repositories

Fetch and merge the latest changes from remotes:

```sh
wok update
```

This will:
- Fetch changes from remote for each repository
- Merge changes into the configured branch
- Commit updated submodule state to the umbrella repo
- Fetch and merge the umbrella repository itself (use `--no-umbrella` to skip)

Use `--no-commit` to stage changes without committing:

```sh
wok update --no-commit
```

### Lock Submodule State

Commit the current submodule commit references:

```sh
wok lock
```

This ensures the umbrella repository captures the exact commit hashes of all submodules.

### Push Changes

Push changes from all repositories:

```sh
# Push repos matching current branch
wok push

# Push all configured repos
wok push --all

# Set upstream for new branches
wok push --all -u
```

### Tag Releases

Create and manage tags across repositories:

```sh
# List tags in all repos on current branch
wok tag

# Create tag in all repos
wok tag --create v1.0.0 --all

# Create signed tag and push
wok tag --create v1.0.0 --all --sign --push

# Tag specific repos
wok tag --create v1.0.0 api frontend
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
wok push

# All configured repos
wok push --all

# Specific repos only
wok push api docs
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

Now `wok push --all` will skip `archived-component`, but `wok push archived-component` will still work.

### Shell Completion

Generate shell completion scripts:

```sh
# Bash
wok completion bash > ~/.local/share/bash-completion/completions/wok

# Zsh
wok completion zsh > ~/.zsh/completions/_wok

# Fish
wok completion fish > ~/.config/fish/completions/wok.fish
```

## Configuration File

The workspace configuration lives in `wok.toml` at the root of your umbrella repository. See the [Wokfile documentation](wokfile.md) for detailed syntax.

## Common Workflows

### Feature Development Workflow

```sh
# Start feature branch
git checkout -b feature-new-api
wok switch --all --create --branch feature-new-api

# Make changes in subrepositories...
# (work in api/, frontend/, etc.)

# Check status
wok status

# Lock state periodically
wok lock

# Push changes
wok push --all -u

# Merge and tag release
git checkout main
git merge feature-new-api
wok tag --create v2.0.0 --all --sign --push
```

### Maintenance Workflow

```sh
# Update all repos to latest
wok update

# Review changes
wok status

# If satisfied, the umbrella repo now has a commit with updated submodules
git push
```

### Adding a New Repository

```sh
# Add as submodule first
git submodule add https://github.com/user/new-repo.git new-repo

# Register with wok
wok add new-repo

# Initialize and switch to current branch
wok switch --create new-repo

# Commit configuration
git add wok.toml .gitmodules new-repo
git commit -m "Add new-repo to workspace"
```

## Troubleshooting

### "Git Wok file not found"

Ensure you're in the umbrella repository root, or use `-f` to specify the path:

```sh
wok -f path/to/wok.toml status
```

### "Branch does not exist"

Use `--create` when switching to a new branch:

```sh
wok switch --all --create --branch new-feature
```

### Merge Conflicts During Update

The `update` command will report conflicts. Resolve them manually in each affected repository, then commit:

```sh
cd affected-repo
# Resolve conflicts
git add .
git commit

cd ..
wok lock  # Update umbrella with resolved state
```

### Authentication Failures During Update/Push

If `wok update` or `wok push` fail with authentication errors, but `git fetch` works from the command line, this typically indicates a difference in how libgit2 (used by wok) handles credentials compared to the git CLI.

**Diagnostic Steps:**

1. **Test authentication explicitly:**
   ```sh
   wok test-auth
   ```
   This will attempt to connect to your remote and show detailed debugging information.

2. **Check SSH agent availability:**
   ```sh
   echo $SSH_AUTH_SOCK
   ssh-add -l
   ```
   If the SSH agent is not running, start it:
   ```sh
   eval $(ssh-agent)
   ssh-add
   ```

3. **Verify SSH key permissions:**
   ```sh
   ls -la ~/.ssh/
   chmod 600 ~/.ssh/id_*
   chmod 644 ~/.ssh/id_*.pub
   ```

4. **Test SSH connection directly:**
   ```sh
   ssh -T git@github.com  # or your git host
   ```

5. **Check credential helper configuration:**
   ```sh
   git config --global credential.helper
   ```
   If not set, you can configure it:
   ```sh
   git config --global credential.helper cache
   ```

**Common Fixes:**

- **SSH agent not accessible:** Ensure `SSH_AUTH_SOCK` is set and the socket file exists. Wok uses libgit2 which requires explicit SSH agent access.
- **SSH keys with passphrase:** Ensure keys are added to the SSH agent with `ssh-add`.
- **Wrong key permissions:** SSH keys should be readable only by you (`chmod 600`).
- **Credential helper mismatch:** Some git credential helpers may not work with libgit2. Try using the `cache` helper.

### Differences Between Git CLI and Wok

Wok uses libgit2 internally, which may handle authentication differently than the git command-line tool:

- libgit2 requires explicit SSH agent access (`SSH_AUTH_SOCK` must be set and accessible)
- Some git credential helpers may not work with libgit2
- Environment variables that work with git may need to be explicitly set for wok
- libgit2 will attempt multiple authentication methods in order:
  1. SSH key from agent
  2. SSH key files from `~/.ssh/` (id_ed25519, id_rsa, id_ecdsa)
  3. Credential helper
  4. Default credentials

If you continue to experience authentication issues after trying these steps, the debug output from `wok test-auth` will help identify which authentication method is failing and why.

### Submodule Not Initialized

Initialize submodules if needed:

```sh
git submodule update --init --recursive
```

## Next Steps

- Read the [CLI Reference](cli.md) for detailed command documentation
- Learn about [Wokfile syntax](wokfile.md) for advanced configuration
- Join the community on [Delta Chat](https://i.delta.chat/#667BD2FB6B122F4138F29A17861B4E257DCDFDB9&a=lig%40countzero.co&g=Git%20Wok&x=0FgEK_cMRZ6NMvG1PAekdJE3&i=9Jn9KZM9tErF-O0k8xvadsn_&s=DyV77Vq3p4y86HX9rRuOMvm2)
