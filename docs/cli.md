# Git Wok Commands Reference

This page documents all available Git Wok commands and their options.

## Global Options

These options apply to all commands.

### -f / --wokfile-path

```sh
wok -f <WOK_FILE_PATH>
wok --wokfile-path <WOK_FILE_PATH>
```

!!! abstract "Default"
    `wok.toml`

Override the default path to [Wokfile](./wokfile.md).

### --help

```sh
wok --help
wok <COMMAND> --help
```

Show the list of possible commands and global options. When used with a specific subcommand, show help for that subcommand.

---

## Workspace Initialization Commands

### init

```sh
wok init
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
wok init
```

### assemble

```sh
wok assemble <DIRECTORY>
```

Assemble a workspace by initializing subrepos and generating the configuration.

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
wok assemble .

# Now you have:
# workspace/.git
# workspace/.gitmodules
# workspace/wok.toml
```

---

## Housekeeping Commands

### status

```sh
wok status [--fetch]
```

Show the status of the umbrella repository and all configured subrepos.

**Options:**

- `--fetch` - Fetch from remotes before comparing local and remote branches (performs network operations)

**What it shows:**

- Current branch of umbrella repository
- Whether umbrella repository has uncommitted changes
- Comparison with remote tracking branch (ahead/behind/diverged/up-to-date)
- Current branch of each subrepo
- Whether each subrepo has uncommitted changes
- Comparison with remote tracking branch for each subrepo

**Note:** By default, status does not perform any network operations. Use `--fetch` to update remote refs before comparison. Without `--fetch`, the comparison is based on the last fetched remote state.

**Example output (without fetch):**

```
On branch 'main', all clean, up to date with 'origin/main'
- 'api' is on branch 'main', all clean, ahead of 'origin/main' by 2 commits
- 'frontend' is on branch 'develop', behind 'origin/develop' by 3 commits
- 'docs' is on branch 'main', all clean
```

**Example with fetch:**

```sh
wok status --fetch
```

Output:

```
On branch 'main', all clean, diverged from 'origin/main' (1 ahead, 2 behind)
- 'api' is on branch 'main', all clean, up to date with 'origin/main'
```

**Remote status indicators:**

- `up to date with 'origin/main'` - Local and remote branches point to the same commit
- `ahead of 'origin/main' by N commit(s)` - Local branch has N commits not yet pushed to remote
- `behind 'origin/main' by N commit(s)` - Remote branch has N commits not yet pulled locally
- `diverged from 'origin/main' (N ahead, M behind)` - Both local and remote have unique commits
- No indicator - No remote tracking branch configured for this branch

---

## Repository Management Commands

### add

```sh
wok add <SUBMODULE_PATH>
```

Add a submodule previously configured in the repository to the [Wokfile](./wokfile.md).

**Prerequisites:**
- Submodule must already be added with `git submodule add`
- Wokfile must exist

**Arguments:**
- `<SUBMODULE_PATH>` - Path to the submodule relative to umbrella repository root

**Example:**
```sh
git submodule add https://github.com/user/component component
wok add component
```

### rm

```sh
wok rm <SUBMODULE_PATH>
```

Remove a submodule from [Wokfile](./wokfile.md) configuration.

**Note:** This only removes the entry from `wok.toml`. It does not remove the git submodule itself. Use `git submodule deinit` and `git rm` to fully remove a submodule.

**Arguments:**
- `<SUBMODULE_PATH>` - Path to the submodule relative to umbrella repository root

**Example:**
```sh
wok rm component
# Then, if desired:
git submodule deinit component
git rm component
```

---

## Branch Management Commands

### switch

```sh
wok switch -b <BRANCH> [OPTIONS] [REPOS]...
```

Switch repositories using a target umbrella branch and its wokfile state.

**Options:**

#### --all

```sh
wok switch -b <BRANCH> --all
```

After reconciling with the target branch’s wokfile, force all non-skipped repos onto the umbrella branch. Repos in the skip list can still be targeted explicitly.

**Note:** This flag replaces the deprecated `head switch` command. Use `wok switch -b <BRANCH> --all` to switch all repositories to the umbrella branch.

#### -c / --create

```sh
wok switch -b <BRANCH> -c
wok switch -b <BRANCH> --create
```

Create the target branch in repositories if it doesn't exist. Without this flag, the command fails if the branch doesn't exist. Existing branches are never recreated.

#### -b / --branch <BRANCH>

```sh
wok switch -b <BRANCH_NAME>
wok switch --branch <BRANCH_NAME>
```

Required. Switch the umbrella repo to this branch first, then load the wokfile from that branch to determine subrepo branches.

#### repos

```sh
wok switch -b <BRANCH> <REPO1> <REPO2> ...
```

Specific repositories to force onto the umbrella branch after the base reconcile. If not provided and `--all` is not used, the command only reconciles repos to the wokfile’s per-repo heads.

**Examples:**
```sh
# Reconcile repos to the wokfile on branch main
wok switch -b main

# Force all repos to the umbrella branch after reconcile
wok switch --all -b main

# Force specific repos to the umbrella branch after reconcile
wok switch -b main api frontend

# Switch umbrella to a specific branch and force all repos to it
wok switch -b develop --all

# Switch to specific branch (short form)
wok switch -b develop --all

# Create and switch to new branch (long form)
wok switch -b feature-new --all --create

# Create and switch to new branch (short form)
wok switch -b feature-new --all -c
```

**Behavior:**
- Switch the umbrella repository to the target branch before loading `wok.toml`
- Reconcile subrepos to the per-repo `head` values in the target branch’s wokfile
- When using `--all` or explicit repo arguments, force those repos to the umbrella branch and update `wok.toml` accordingly
- Commit submodule pointer changes **and** any wokfile changes together
- Skip repos with `switch` in their `skip_for` list (unless explicitly targeted)

**Commit Message Format:**

When submodules are switched, the commit message shows which repos changed:
```
Switch and lock submodule state

Switched submodules:
- api: feature-branch
- frontend: feature-branch
- docs: feature-branch
```

---

## Synchronization Commands

### lock

```sh
wok lock
```

Lock the current submodule state by committing submodule commit references.

**What it does:**
- Ensure each repo is on its configured branch
- Add all submodule entries to the git index
- Compare the current state with the previous commit
- **Only commit if submodules have changed** (no-op if nothing changed)
- Create a commit with updated commit hashes of changed submodules

**Commit Message Format:**

When submodules have changed, the lock command creates a descriptive commit message:
- Summary header: "Lock submodule state"
- List of changed submodules with their latest commit messages (truncated to 50 characters)

Example commit message:
```
Lock submodule state

Changed submodules:
- api: Add new authentication endpoint for OAuth2 flow
- frontend: Update dashboard UI with new chart compone...
- docs: Fix typo in installation guide
```

**Output Messages:**
- `"Locked submodule state"` - When changes were committed
- `"No submodule changes detected; nothing to lock"` - When no changes were found

**Use case:**
Capturing a snapshot of the workspace state after making changes in subrepos.

**Example:**
```sh
# After making changes in api/ and frontend/
cd api && git commit -am "Update API" && cd ..
cd frontend && git commit -am "Update UI" && cd ..

# Lock the state - creates commit with summary
wok lock
# Output: Locked submodule state

# Running lock again with no changes
wok lock
# Output: No submodule changes detected; nothing to lock

# Umbrella repo now has a commit with updated submodule pointers
git log -1
# Shows:
# Lock submodule state
#
# Changed submodules:
# - api: Update API
# - frontend: Update UI
```

### update

```sh
wok update [OPTIONS]
```

Update submodules by fetching and merging latest changes from remotes.

**What it does:**
- Switch each repo to its configured branch
- Fetch changes from the remote
- Merge or rebase changes into the local branch (respects `pull.rebase` and `branch.<name>.rebase` config)
- Stage submodule updates in the umbrella repository
- Commit the updated state (unless `--no-commit` is used)

**Options:**

#### --no-commit

```sh
wok update --no-commit
```

Stage submodule updates without creating a commit in the umbrella repository.

#### --umbrella / --no-umbrella

```sh
wok update --no-umbrella
wok update --umbrella
```

Control whether the umbrella repository is fetched and merged alongside subrepos. Enabled by default; use `--no-umbrella` to skip updating the umbrella repo.

**Behavior:**
- Skip repos with `update` in their `skip_for` list
- Report merge conflicts if any occur
- Do not commit umbrella repo if conflicts are detected
- Include the umbrella repository in the update process unless `--no-umbrella` is provided

**Examples:**
```sh
# Update and commit
wok update

# Update but review before committing
wok update --no-commit
git diff --staged
git commit -m "Update submodules"

# Update only subrepos, skip umbrella
wok update --no-umbrella
```

**Example output:**
```
Updating submodules...
- 'umbrella': already up to date on 'main' (12345678)
- 'api': fast-forwarded 'main' to a1b2c3d4
- 'frontend': rebased 'develop' to e5f6g7h8
- 'docs': already up to date on 'main' (i9j0k1l2)
Updated submodule state committed
```

**Commit Message Format:**

When submodules are updated, the commit message shows what changed:
```
Update submodules to latest

Updated submodules:
- api: main to a1b2c3d4
- frontend: develop to e5f6g7h8
- docs: main to i9j0k1l2
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
wok push [OPTIONS] [REPOS]...
```

Push changes from configured repositories to their remotes.

**Options:**

#### -u / --set-upstream

```sh
wok push -u
wok push --set-upstream
```

Set upstream tracking for new branches.

#### --all

```sh
wok push --all
```

Act on all configured repos, respecting `skip_for` settings.

#### --branch <BRANCH>

```sh
wok push --branch <BRANCH_NAME>
```

Push the specified branch instead of the current umbrella repository branch.

#### --umbrella / --no-umbrella

```sh
wok push --no-umbrella
wok push --umbrella
```

Control whether the umbrella repository is included in the push. The umbrella repo is included by default; pass `--no-umbrella` to skip it.

#### repos

```sh
wok push <REPO1> <REPO2> ...
```

Specific repositories to push. If not provided and `--all` is not used, pushes repos matching the current umbrella branch.

**Behavior:**
- Check remote state before pushing to avoid unnecessary operations
- Skip push entirely if the remote branch already matches the local branch
- Skip repos with `push` in their `skip_for` list (unless explicitly targeted)
- Report which repos were pushed successfully
- Handle "up to date" and error cases gracefully
- Include the umbrella repository by default so workspace-level changes are pushed alongside subrepos (unless `--no-umbrella` is specified)

**Examples:**
```sh
# Push repos on current branch
wok push

# Push all repos
wok push --all

# Push specific repos
wok push api docs

# Push and set upstream
wok push --all -u

# Push specific branch
wok push --all --branch release/v2

# Push only subrepos, skip umbrella
wok push --all --no-umbrella
```

---

## Release Management Commands

### tag

```sh
wok tag [SUBCOMMAND] [OPTIONS] [ARGS]...
```

Create, list, and push tags across repositories. The command supports three subcommands: `create`, `list`, and `push`.

**Subcommands:**

- `create <TAG>` - Create a new tag
- `list` - List existing tags
- `push` - Push tags to remote repositories

**Backward Compatibility:**

For convenience, `wok tag` supports implicit subcommand behavior:
- `wok tag` (no arguments) → Lists tags (equivalent to `wok tag list`)
- `wok tag v1.0` → Creates tag (equivalent to `wok tag create v1.0`)

**Common Options (apply to all subcommands):**

#### --all

```sh
wok tag --all create v1.0.0
wok tag --all list
wok tag --all push
```

Act on all configured repos, respecting `skip_for` settings.

#### --umbrella / --no-umbrella

```sh
wok tag --no-umbrella create v1.0.0
wok tag --umbrella list
```

Control whether the umbrella repository participates in operations. Enabled by default; use `--no-umbrella` to limit operations to subrepos.

---

### tag create

```sh
wok tag create <TAG_NAME> [OPTIONS] [REPOS]...
```

Create a new tag with the specified name in repositories.

**Arguments:**

- `<TAG_NAME>` - Name of the tag to create (required)
- `[REPOS]...` - Specific repositories to tag (optional)

**Options:**

#### -s / --sign

```sh
wok tag create v1.0.0 -s
wok tag create v1.0.0 --sign
```

Sign the tag with GPG. Requires GPG to be configured. Creates an annotated tag.

#### -m / --message <MESSAGE>

```sh
wok tag create v1.0.0 -m "Release version 1.0.0"
wok tag create v1.0.0 --message "Release version 1.0.0"
```

Add a message to create an annotated tag. If not provided with `--sign`, lightweight tags are created by default.

When combined with `--sign`, the message is included in the signed annotated tag. When used without `--sign`, an unsigned annotated tag is created with the provided message.

**Note:** Annotated tags (created with `--sign` or `--message`) include metadata like tagger name, email, and timestamp. Lightweight tags are simple pointers to commits without additional metadata.

#### -u / --updated

```sh
wok tag --all create v1.0.0 --updated
wok tag --all create v1.0.0 -u
wok tag create v1.0.0 -u
```

Only create tags in repositories where the current HEAD commit has no existing tags. This is useful for tagging only repos with new changes to release.

**Behavior:**
- Filters out subrepos where the current commit already has one or more tags
- The umbrella repo is **not** filtered by this flag - it's still controlled by `--umbrella/--no-umbrella`
- Useful for incremental releases where only some components have changed

**Example workflow:**
```sh
# After making changes to some repos, tag only the updated ones
wok tag --all create v1.2.0 --updated

# This will:
# - Always tag the umbrella repo (unless --no-umbrella is used)
# - Only tag subrepos where HEAD has no tags (i.e., new commits since last tag)
```

**Behavior:**
- Skip repos with `tag` in their `skip_for` list (unless explicitly targeted)
- Report creation status for each repo
- Handle tag conflicts gracefully (reports if tag already exists)
- Include the umbrella repository by default (disable with `--no-umbrella`)

**Examples:**

```sh
# Create tag in repos on current branch
wok tag create v1.0.0

# Create in all repos
wok tag --all create v1.0.0

# Create tag with custom message
wok tag --all create v1.0.0 --message "Release version 1.0.0"

# Short form with message
wok tag --all create v1.0.0 -m "Release version 1.0.0"

# Create signed tag
wok tag --all create v1.0.0 --sign

# Create signed tag with message (short form)
wok tag --all create v1.0.0 -s -m "Signed release v1.0.0"

# Create in specific repos
wok tag create v2.0.0 api docs

# Create only in repos with uncommitted changes (no tags on HEAD)
wok tag --all create v1.2.0 --updated

# Create only in updated repos, short form
wok tag --all create v1.2.0 -u

# Create only in subrepos, skip umbrella
wok tag --all --no-umbrella create v2.0.0

# Implicit create (backward compatible)
wok tag --all v1.0.0 --sign
wok tag --all v1.0.0 -s -m "Release version 1.0.0"
wok tag v2.0.0 api docs
```

**Example output:**
```
Creating tag 'v1.0.0' in 3 repositories...
- 'umbrella': created tag 'v1.0.0'
- 'api': created tag 'v1.0.0'
- 'frontend': created tag 'v1.0.0'
Successfully processed 3 repositories
```

---

### tag list

```sh
wok tag list [OPTIONS] [REPOS]...
```

List existing tags in repositories.

**Arguments:**

- `[REPOS]...` - Specific repositories to list tags from (optional)

**Behavior:**
- Skip repos with `tag` in their `skip_for` list (unless explicitly targeted)
- Display all tags for each repository
- Include the umbrella repository by default (disable with `--no-umbrella`)

**Examples:**

```sh
# List tags in repos on current branch
wok tag list

# List tags in all repos
wok tag --all list

# List tags in specific repos
wok tag list api frontend

# List only from subrepos, skip umbrella
wok tag --all --no-umbrella list

# Implicit list (backward compatible)
wok tag
wok tag --all
wok tag api frontend
```

**Example output:**
```
Listing tags in 3 repositories...
- 'umbrella': v0.9.0, v1.0.0, v1.1.0
- 'api': v1.0.0, v1.1.0
- 'frontend': v1.0.0
Successfully processed 3 repositories
```

---

### tag push

```sh
wok tag push [OPTIONS] [REPOS]...
```

Push tags to remote repositories.

**Arguments:**

- `[REPOS]...` - Specific repositories to push tags from (optional)

**Behavior:**
- Push all local tags to the remote
- Skip tags that already exist on remote with the same commit
- Report push status for each repository
- Include the umbrella repository by default (disable with `--no-umbrella`)

**Examples:**

```sh
# Push tags from repos on current branch
wok tag push

# Push tags from all repos
wok tag --all push

# Push tags from specific repos
wok tag push api frontend

# Push only from subrepos, skip umbrella
wok tag --all --no-umbrella push

# Create and push in one workflow (two commands)
wok tag --all create v1.0.0 --sign
wok tag --all push
```

**Example output:**
```
Pushing tags to remotes...
- 'umbrella': pushed tags
- 'api': pushed tags
- 'frontend': no tags to push
Successfully processed 3 repositories
```

---

## Utility Commands

### completion

```sh
wok completion [SHELL]
```

Generate shell completion script for the specified shell.

**Arguments:**
- `[SHELL]` - Shell type: `bash`, `fish`, or `zsh` (default: `bash`)

**Installation:**

Bash:
```sh
wok completion bash > ~/.local/share/bash-completion/completions/wok
```

Zsh:
```sh
wok completion zsh > ~/.zsh/completions/_wok
# Add to .zshrc if not already there:
# fpath=(~/.zsh/completions $fpath)
# autoload -Uz compinit && compinit
```

Fish:
```sh
wok completion fish > ~/.config/fish/completions/wok.fish
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
wok push archived-component  # This works even with skip_for = ["push"]
```
