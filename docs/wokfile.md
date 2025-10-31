# Wokfile (`wok.toml`)

Git Wok stores the configuration for subrepos in a file called `wok.toml`.

## File Location

The default path is `wok.toml` in the root directory of the umbrella repository.

You can override this with the `-f` / `--wokfile-path` option:

```sh
git-wok -f custom-config.toml status
```

## File Format

The Wokfile uses **TOML** (Tom's Obvious, Minimal Language) syntax, the same format used by Cargo.toml and many other Rust tools.

## Basic Structure

Here's a minimal example:

```toml
version = "1.0-experimental"

[[repo]]
path = "api"
head = "main"

[[repo]]
path = "frontend"
head = "develop"
```

## Top-Level Fields

### version

**Type:** String
**Required:** Yes
**Current value:** `"1.0-experimental"`

Specifies the Wokfile schema version. This allows Git Wok to handle configuration format changes gracefully in the future.

**Example:**
```toml
version = "1.0-experimental"
```

### repo

**Type:** Array of Repo Objects
**Required:** Yes (but can be empty)

List of configured subrepos. Each entry is a TOML table specifying one subrepo.

## Repo Object Fields

Each `[[repo]]` table represents one subrepo in your workspace.

### path

**Type:** String
**Required:** Yes

Path to the subrepo relative to the umbrella repository root. Must match the submodule path in `.gitmodules`.

**Example:**
```toml
[[repo]]
path = "api"
head = "main"

[[repo]]
path = "packages/frontend"
head = "main"
```

### head

**Type:** String
**Required:** Yes

The branch name that this subrepo should track. Used by commands like `switch`, `update`, and `lock` to determine which branch to operate on.

**Example:**
```toml
[[repo]]
path = "api"
head = "main"

[[repo]]
path = "experimental"
head = "develop"
```

### skip_for

**Type:** Array of Strings
**Required:** No (defaults to empty)

List of command names that should skip this repo when using `--all` flag. The repo can still be targeted explicitly.

**Commands that honor skip_for:**
- `switch`
- `push`
- `tag`
- `update`

**Example:**
```toml
[[repo]]
path = "archived-component"
head = "main"
skip_for = ["push", "update", "tag"]

[[repo]]
path = "experimental"
head = "develop"
skip_for = ["push", "tag"]
```

With this configuration:
- `git-wok push --all` will skip `archived-component`
- `git-wok push archived-component` will still work
- `git-wok update` will skip `archived-component`
- `git-wok switch --all` will process `archived-component` normally

## Complete Examples

### Simple Workspace

A basic workspace with three components on the same branch:

```toml
version = "1.0-experimental"

[[repo]]
path = "api"
head = "main"

[[repo]]
path = "frontend"
head = "main"

[[repo]]
path = "docs"
head = "main"
```

### Multi-Branch Workspace

Different components on different branches:

```toml
version = "1.0-experimental"

[[repo]]
path = "api"
head = "main"

[[repo]]
path = "frontend"
head = "develop"

[[repo]]
path = "experimental-feature"
head = "feature/new-arch"

[[repo]]
path = "docs"
head = "main"
```

### Workspace with Skip Lists

Some components excluded from certain operations:

```toml
version = "1.0-experimental"

# Active development repos
[[repo]]
path = "api"
head = "main"

[[repo]]
path = "frontend"
head = "main"

# Archived component - don't push or update
[[repo]]
path = "legacy-api"
head = "main"
skip_for = ["push", "update", "tag"]

# Experimental component - don't push or tag
[[repo]]
path = "experimental"
head = "develop"
skip_for = ["push", "tag"]

# Documentation - update but don't tag
[[repo]]
path = "docs"
head = "main"
skip_for = ["tag"]
```

### Monorepo with Nested Paths

Repos organized in subdirectories:

```toml
version = "1.0-experimental"

[[repo]]
path = "services/api"
head = "main"

[[repo]]
path = "services/auth"
head = "main"

[[repo]]
path = "services/storage"
head = "main"

[[repo]]
path = "clients/web"
head = "main"

[[repo]]
path = "clients/mobile"
head = "main"

[[repo]]
path = "shared/models"
head = "main"
```

## Automatic Management

In most cases, you won't need to edit `wok.toml` manually. Git Wok commands update it automatically:

- `git-wok init` - Creates the file and populates it with existing submodules
- `git-wok assemble` - Creates the file with discovered repos
- `git-wok add` - Adds a new repo entry
- `git-wok rm` - Removes a repo entry
- `git-wok switch --all` - Updates all `head` fields to match umbrella branch

## Manual Editing

You may want to manually edit `wok.toml` to:

1. Add `skip_for` entries to exclude repos from bulk operations
2. Change the `head` branch for a repo (though `switch` is preferred)
3. Reorder entries for readability

After manual edits, verify the configuration is valid:

```sh
git-wok status
```

## Configuration Validation

Git Wok validates the configuration when loading. Common errors:

**Error: "Cannot parse the wok file"**
- TOML syntax error
- Check for missing quotes, commas, or brackets
- Validate at [TOML Lint](https://www.toml-lint.com/)

**Error: "unknown field"**
- You used a field name that doesn't exist
- Check spelling: `head` not `ref`, `path` not `name`

**Error: "missing field"**
- A required field is missing
- Ensure each repo has both `path` and `head`

## Version Control

The `wok.toml` file should be committed to your umbrella repository:

```sh
git add wok.toml
git commit -m "Add workspace configuration"
```

This allows team members to clone the umbrella repository and immediately have the workspace configuration.

## Comparison to Package Managers

| Package Manager | Config File | Lock File |
|----------------|-------------|-----------|
| Cargo | `Cargo.toml` | `Cargo.lock` |
| npm | `package.json` | `package-lock.json` |
| Poetry | `pyproject.toml` | `poetry.lock` |
| **Git Wok** | `wok.toml` | Committed submodules in `.git/modules` and `.gitmodules` |

Git Wok's `wok.toml` is analogous to `Cargo.toml` or `package.json`, while the committed submodule state in the umbrella repository acts as the lock file.

## Migration Notes

### From wok.yaml (Pre-1.0)

If you have an old `wok.yaml` file from an early version:

1. The file format changed from YAML to TOML
2. The field name changed from `ref` to `head`
3. Convert manually or reinitialize:

```sh
# Backup old config
mv wok.yaml wok.yaml.backup

# Reinitialize (preserves submodules)
git-wok init

# Manually re-add skip_for lists if needed
```

## Further Reading

- [TOML Specification](https://toml.io/)
- [Git Submodules Documentation](https://git-scm.com/book/en/v2/Git-Tools-Submodules)
- [CLI Reference](cli.md) for commands that interact with the Wokfile
