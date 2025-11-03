# Git Wok Commands Reference

This page documents all available Git Wok commands and their options.

## Global Options

These options apply to all commands.

### -f / --wokfile-path

```sh
git-wok -f <WOK_FILE_PATH>
git-wok --wokfile-path <WOK_FILE_PATH>
```

!!! abstract "Default"
    `wok.toml`

Overrides default path to [Wokfile](./wokfile.md).

### --help

```sh
git-wok --help
git-wok <COMMAND> --help
```

Shows list of possible commands and global options. When used with a specific subcommand, shows help for that subcommand.

---

## Workspace Initialization Commands

### init

```sh
git-wok init
```

Initialize Git Wok configuration in an umbrella repository.

**What it does:**
- Creates [Wokfile](./wokfile.md) (`wok.toml`) in the repository
- Introspects existing submodules and adds them to the configuration
- Optionally switches submodules to match the current branch

**Prerequisites:**
- Must be run in a Git repository
- Repository should already be initialized with `git init`

**Fails if:**
- Wokfile already exists in the repository
- Not inside a git repository

**Example:**
```sh
mkdir my-workspace && cd my-workspace
git init
git submodule add https://github.com/user/api api
git-wok init
```

### assemble

```sh
git-wok assemble <DIRECTORY>
```

Assemble a workspace by initializing subrepos and generating configuration.

**What it does:**
- Initializes the umbrella repository if needed
- Discovers all subdirectories containing git repositories
- Registers them as git submodules
- Preserves existing remote URLs
- Creates the Wokfile with all discovered repositories

**Arguments:**
- `<DIRECTORY>` - Path to the workspace directory to assemble

**Use case:**
Converting a directory with multiple independent git repositories into a managed workspace.

**Example:**
```sh
# You have a directory structure:
# workspace/
#   api/       (git repo)
#   frontend/  (git repo)
#   docs/      (git repo)

cd workspace
git-wok assemble .

# Now you have:
# workspace/.git
# workspace/.gitmodules
# workspace/wok.toml
```

---

## Housekeeping Commands

### status

```sh
git-wok status
```

Show the status of the umbrella repository and all configured subrepos.

**What it shows:**
- Current branch of umbrella repository
- Whether umbrella repository has uncommitted changes
- Current branch of each subrepo
- Whether each subrepo has uncommitted changes

**Example output:**
```
On branch 'main', all clean
- 'api' is on branch 'main', all clean
- 'frontend' is on branch 'develop'
- 'docs' is on branch 'main', all clean
```

---

## Repository Management Commands

### add

```sh
git-wok add <SUBMODULE_PATH>
```

Add a submodule previously configured in the repository to [Wokfile](./wokfile.md).

**Prerequisites:**
- Submodule must already be added with `git submodule add`
- Wokfile must exist

**Arguments:**
- `<SUBMODULE_PATH>` - Path to the submodule relative to umbrella repository root

**Example:**
```sh
git submodule add https://github.com/user/component component
git-wok add component
```

### rm

```sh
git-wok rm <SUBMODULE_PATH>
```

Remove a submodule from [Wokfile](./wokfile.md) configuration.

**Note:** This only removes the entry from `wok.toml`. It does not remove the git submodule itself. Use `git submodule deinit` and `git rm` to fully remove a submodule.

**Arguments:**
- `<SUBMODULE_PATH>` - Path to the submodule relative to umbrella repository root

**Example:**
```sh
git-wok rm component
# Then, if desired:
git submodule deinit component
git rm component
```

---

## Branch Management Commands

### switch

```sh
git-wok switch [OPTIONS] [REPOS]...
```

Switch repositories to a specified branch with fine-grained control.

**Options:**

#### --all

```sh
git-wok switch --all
```

Act on all configured repos, respecting `skip_for` settings. Repos in the skip list can still be targeted explicitly.

**Note:** This flag replaces the deprecated `head switch` command. Use `git-wok switch --all` to switch all repositories to the umbrella's current branch.

#### --create

```sh
git-wok switch --create
```

Create the target branch in repositories if it doesn't exist. Without this flag, the command fails if the branch doesn't exist.

#### --branch <BRANCH>

```sh
git-wok switch --branch <BRANCH_NAME>
```

Use the specified branch name instead of the current umbrella repository branch.

#### repos

```sh
git-wok switch <REPO1> <REPO2> ...
```

Specific repositories to switch. If not provided and `--all` is not used, switches repos matching the current umbrella branch.

**Examples:**
```sh
# Switch repos matching current branch
git checkout main
git-wok switch

# Switch all repos
git-wok switch --all

# Switch specific repos
git-wok switch api frontend

# Switch to specific branch
git-wok switch --all --branch develop

# Create and switch to new branch
git-wok switch --all --create --branch feature-new
```

**Behavior:**
- Updates the Wokfile configuration to reflect new branch assignments
- Commits submodule state changes to umbrella repository
- Skips repos with `switch` in their `skip_for` list (unless explicitly targeted)

---

## Synchronization Commands

### lock

```sh
git-wok lock
```

Lock the current submodule state by committing submodule commit references.

**What it does:**
- Ensures each repo is on its configured branch
- Adds all submodule entries to the git index
- Commits the current commit hashes of all submodules to the umbrella repository

**Use case:**
Capturing a snapshot of the workspace state after making changes in subrepos.

**Example:**
```sh
# After making changes in api/ and frontend/
cd api && git commit -am "Update API" && cd ..
cd frontend && git commit -am "Update UI" && cd ..

# Lock the state
git-wok lock

# Umbrella repo now has a commit with updated submodule pointers
```

### update

```sh
git-wok update [OPTIONS]
```

Update submodules by fetching and merging latest changes from remotes.

**What it does:**
- Switches each repo to its configured branch
- Fetches changes from the remote
- Merges or rebases changes into the local branch (respects `pull.rebase` and `branch.<name>.rebase` config)
- Stages submodule updates in the umbrella repository
- Commits the updated state (unless `--no-commit` is used)

**Options:**

#### --no-commit

```sh
git-wok update --no-commit
```

Stage submodule updates without creating a commit in the umbrella repository.

**Behavior:**
- Skips repos with `update` in their `skip_for` list
- Reports merge conflicts if any occur
- Does not commit umbrella repo if conflicts are detected

**Examples:**
```sh
# Update and commit
git-wok update

# Update but review before committing
git-wok update --no-commit
git diff --staged
git commit -m "Update submodules"
```

**Example output:**
```
Updating submodules...
- 'api': fast-forwarded 'main' to a1b2c3d4
- 'frontend': rebased 'develop' to e5f6g7h8
- 'docs': already up to date on 'main' (i9j0k1l2)
Updated submodule state committed
```

**Pull Strategy:**

`wok update` respects your git configuration for pull strategy:

- By default, or when `pull.rebase = false`, changes are merged
- When `pull.rebase = true`, changes are rebased
- Per-branch configuration with `branch.<name>.rebase` takes precedence over global `pull.rebase`

Example configurations:
```sh
# Set global preference to rebase
git config --global pull.rebase true

# Set specific branch to use merge in a subrepo
cd api/
git config branch.main.rebase false
```

**Note:** Interactive rebase (`pull.rebase = interactive`) and preserve-merges rebase (`pull.rebase = merges`) are treated as standard rebase in the current implementation.

---

## Remote Operations Commands

### push

```sh
git-wok push [OPTIONS] [REPOS]...
```

Push changes from configured repositories to their remotes.

**Options:**

#### -u / --set-upstream

```sh
git-wok push -u
git-wok push --set-upstream
```

Set upstream tracking for new branches.

#### --all

```sh
git-wok push --all
```

Act on all configured repos, respecting `skip_for` settings.

#### --branch <BRANCH>

```sh
git-wok push --branch <BRANCH_NAME>
```

Push the specified branch instead of the current umbrella repository branch.

#### --umbrella / --no-umbrella

```sh
git-wok push --no-umbrella
git-wok push --umbrella
```

Control whether the umbrella repository is included in the push. The umbrella repo is included by default; pass `--no-umbrella` to skip it.

#### repos

```sh
git-wok push <REPO1> <REPO2> ...
```

Specific repositories to push. If not provided and `--all` is not used, pushes repos matching the current umbrella branch.

**Behavior:**
- Checks remote state before pushing to avoid unnecessary operations
- Skips push entirely if the remote branch already matches the local branch
- Skips repos with `push` in their `skip_for` list (unless explicitly targeted)
- Reports which repos were pushed successfully
- Handles "up to date" and error cases gracefully
- Includes the umbrella repository by default so workspace-level changes are pushed alongside subrepos (unless `--no-umbrella` is specified)

**Examples:**
```sh
# Push repos on current branch
git-wok push

# Push all repos
git-wok push --all

# Push specific repos
git-wok push api docs

# Push and set upstream
git-wok push --all -u

# Push specific branch
git-wok push --all --branch release/v2

# Push only subrepos, skip umbrella
git-wok push --all --no-umbrella
```

---

## Release Management Commands

### tag

```sh
git-wok tag [OPTIONS] [TAG] [REPOS]...
```

Create, list, sign, and push tags across repositories.

**Modes:**

1. **List tags**: When no tag name is provided
2. **Create tag**: When `--create` is used or tag name is provided

**Options:**

#### --create <TAG>

```sh
git-wok tag --create <TAG_NAME>
```

Create a new tag with the specified name.

#### --sign

```sh
git-wok tag --sign
```

Sign the tag with GPG. Requires GPG to be configured.

#### --push

```sh
git-wok tag --push
```

Push tags to remote repositories after creating them.

#### --all

```sh
git-wok tag --all
```

Act on all configured repos, respecting `skip_for` settings.

#### --umbrella / --no-umbrella

```sh
git-wok tag --no-umbrella
git-wok tag --umbrella
```

Control whether the umbrella repository participates in listing or tagging. Enabled by default; use `--no-umbrella` to limit operations to subrepos.

#### Positional Arguments

The command accepts flexible positional argument formats:

- `git-wok tag` - List tags in repos on current branch
- `git-wok tag <TAG>` - List tag in repos on current branch matching `<TAG>`
- `git-wok tag <TAG> <REPO>...` - List tag in specific repos
- `git-wok tag --create <TAG>` - Create tag in repos on current branch
- `git-wok tag --all <TAG>` - When listing with `--all`, interpret first positional arg as tag

**Behavior:**
- Skips repos with `tag` in their `skip_for` list (unless explicitly targeted)
- Reports existing tags or creation status for each repo
- Handles tag conflicts gracefully
- Includes the umbrella repository in listing, creation, and push flows by default (disable with `--no-umbrella`)

**Examples:**

List tags:
```sh
# List tags in repos on current branch
git-wok tag

# List tags in all repos
git-wok tag --all

# List tags in specific repos
git-wok tag api frontend
```

Create tags:
```sh
# Create tag in repos on current branch
git-wok tag --create v1.0.0

# Create in all repos
git-wok tag --create v1.0.0 --all

# Create signed tag
git-wok tag --create v1.0.0 --all --sign

# Create, sign, and push
git-wok tag --create v1.0.0 --all --sign --push

# Create in specific repos
git-wok tag --create v2.0.0 api docs

# Create only in subrepos, skip umbrella
git-wok tag --create v2.0.0 --all --no-umbrella
```

Alternative syntax (positional tag argument):
```sh
# These work similarly to --create
git-wok tag v1.0.0 --all --sign --push
git-wok tag v2.0.0 api docs
```

**Example output:**
```
Creating tag 'v1.0.0' in 3 repositories...
- 'api': created tag 'v1.0.0'
- 'frontend': created tag 'v1.0.0'
- 'docs': tag 'v1.0.0' already exists
Successfully processed 3 repositories
```

---

## Utility Commands

### completion

```sh
git-wok completion [SHELL]
```

Generate shell completion script for the specified shell.

**Arguments:**
- `[SHELL]` - Shell type: `bash`, `fish`, or `zsh` (default: `bash`)

**Installation:**

Bash:
```sh
git-wok completion bash > ~/.local/share/bash-completion/completions/git-wok
```

Zsh:
```sh
git-wok completion zsh > ~/.zsh/completions/_git-wok
# Add to .zshrc if not already there:
# fpath=(~/.zsh/completions $fpath)
# autoload -Uz compinit && compinit
```

Fish:
```sh
git-wok completion fish > ~/.config/fish/completions/git-wok.fish
```

---

## Command Categories Summary

### Workspace Setup
- `init` - Initialize Wokfile in existing repo with submodules
- `assemble` - Create workspace from directory of repos

### Daily Operations
- `status` - Check workspace status
- `switch` - Change branches with options (use `--all` for quick branch sync)
- `lock` - Capture current state
- `update` - Fetch and merge from remotes

### Repository Management
- `add` - Add submodule to config
- `rm` - Remove submodule from config

### Remote Operations
- `push` - Push changes to remotes

### Release Operations
- `tag` - Tag, sign, and push releases

### Utilities
- `completion` - Generate shell completions

---

## Selective Targeting

Most commands support three targeting strategies:

1. **Branch-based (default)**: Operate on repos whose configured branch matches the umbrella's current branch
2. **All repos (`--all`)**: Operate on all configured repos (respecting skip lists)
3. **Explicit**: Specify repo paths as arguments

This allows fine-grained control over which repositories are affected by each operation.

## Skip Lists

The `skip_for` field in `wok.toml` allows excluding repos from bulk operations:

```toml
[[repo]]
path = "archived-component"
head = "main"
skip_for = ["push", "update", "tag"]
```

Commands that honor skip lists: `switch`, `push`, `tag`, `update`

Repos in skip lists can still be targeted explicitly:
```sh
git-wok push archived-component  # This works even with skip_for = ["push"]
```
