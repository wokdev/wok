# Scripts Directory

This directory contains utility scripts for maintaining the Wok project.

## Available Scripts

### bump-version.sh

Updates the version number in both `Cargo.toml` and `pyproject.toml` to ensure consistency across the project.

#### Purpose

This script simplifies version management by:

1. Validating the provided version number follows semantic versioning
2. Updating the version in `Cargo.toml`
3. Updating the version in `pyproject.toml`
4. Verifying both files have the same version after update

#### Prerequisites

- **sed**: Standard Unix text processing tool (pre-installed on Linux/macOS)

#### Usage

##### Basic Usage

Bump the version to a new number:

```bash
./scripts/bump-version.sh 1.0.0
```

##### Pre-release Versions

Update to a pre-release version:

```bash
./scripts/bump-version.sh 1.0.0-beta.5
./scripts/bump-version.sh 2.0.0-rc.1
./scripts/bump-version.sh 1.5.0-alpha.3
```

##### Build Metadata

Include build metadata (following semver):

```bash
./scripts/bump-version.sh 1.0.0+20231020
./scripts/bump-version.sh 1.0.0-beta.4+abc123
```

##### Dry Run Mode

Preview what changes would be made without modifying files:

```bash
./scripts/bump-version.sh --dry-run 1.0.0
```

This is useful for:
- Verifying the version format is valid
- Checking current versions in both files
- Testing the script before making actual changes

#### Command-Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Show help message and exit |
| `--dry-run` | | Preview changes without modifying files |

#### What the Script Does

1. **Version Validation**
   - Checks that the provided version follows semantic versioning format
   - Accepts: `X.Y.Z`, `X.Y.Z-prerelease`, `X.Y.Z-prerelease+build`

2. **Prerequisites Check**
   - Verifies `Cargo.toml` exists
   - Verifies `pyproject.toml` exists
   - Checks that `sed` is available

3. **Show Current Versions**
   - Displays the current version in `Cargo.toml`
   - Displays the current version in `pyproject.toml`
   - Warns if versions are mismatched

4. **Update Files**
   - Updates the first `version = "..."` line in `Cargo.toml`
   - Updates the first `version = "..."` line in `pyproject.toml`
   - Works on both Linux and macOS

5. **Verify Changes**
   - Confirms both files were updated
   - Ensures versions match after the update

#### Workflow Integration

##### Release Process

Typical workflow for creating a new release:

```bash
# 1. Update version
./scripts/bump-version.sh 1.0.0

# 2. Review changes
git diff Cargo.toml pyproject.toml

# 3. Run tests to ensure everything still works
cargo test --all

# 4. Commit the version bump
git add Cargo.toml pyproject.toml
git commit -m "Bump version to 1.0.0"

# 5. Create a git tag
git tag v1.0.0

# 6. Push changes and tag
git push origin main
git push origin v1.0.0
```

##### Automated CI Release

When you push a tag, the CI pipeline automatically:

1. Builds binaries for all supported platforms:
   - Linux (x86_64 and ARM64)
   - macOS (x86_64 Intel and ARM64 Apple Silicon)
   - Windows (x86_64)

2. Generates SHA256 checksums for all artifacts

3. Creates a Codeberg release with the tag name

4. Uploads all binaries and checksums to the release

**Note**: The CI pipeline requires a `codeberg_token` secret to be configured in repository settings.

**Monitor the release**:
- View pipeline status: https://ci.codeberg.org/repos/12553
- Check releases: https://codeberg.org/wok/wok/releases

For more details, see [docs/releases.md](../docs/releases.md).

##### Manual Release (if CI fails)

If the automated CI release fails or you need to publish manually:

```bash
# Build release binaries manually
cargo build --release

# Create release on Codeberg UI
# Upload target/release/wok manually
```

##### Pre-release Workflow

For beta or release candidate versions:

```bash
# Bump to beta version
./scripts/bump-version.sh 1.0.0-beta.5

# Test and verify
cargo test --all

# Commit and tag
git add Cargo.toml pyproject.toml
git commit -m "Bump version to 1.0.0-beta.5"
git tag v1.0.0-beta.5
git push origin main --tags
```

#### Troubleshooting

##### "Invalid version format"

**Problem**: The version doesn't follow semantic versioning.

**Solution**: Use the format `X.Y.Z` or `X.Y.Z-prerelease`:
```bash
# Good
./scripts/bump-version.sh 1.0.0
./scripts/bump-version.sh 1.0.0-beta.4

# Bad
./scripts/bump-version.sh 1.0      # Missing patch version
./scripts/bump-version.sh v1.0.0   # Don't include 'v' prefix
./scripts/bump-version.sh 1.0.0_beta  # Use hyphen, not underscore
```

##### "Cargo.toml not found"

**Problem**: The script can't find the configuration files.

**Solution**: Run the script from the repository root:
```bash
cd /path/to/wok
./scripts/bump-version.sh 1.0.0
```

##### "Version mismatch detected"

**Problem**: `Cargo.toml` and `pyproject.toml` have different versions before the bump.

**Action**: This is just a warning. The script will update both files to the new version, ensuring they match afterward.

##### "Permission denied"

**Problem**: The script doesn't have execute permissions.

**Solution**: Make the script executable:
```bash
chmod +x scripts/bump-version.sh
```

#### Script Output

The script provides color-coded output for easy reading:

- ✓ **Green**: Successful operations
- ✗ **Red**: Errors (will exit)
- ⚠ **Yellow**: Warnings
- ℹ **Blue**: Informational messages

Example output:

```
==> Checking prerequisites
✓ Found Cargo.toml
✓ Found pyproject.toml

==> Current versions
ℹ Cargo.toml:     1.0.0-beta.4
ℹ pyproject.toml: 1.0.0-beta.4

==> Bumping version to 1.0.0
✓ Updated Cargo.toml
✓ Updated pyproject.toml

==> Verifying changes
ℹ Cargo.toml:     1.0.0
ℹ pyproject.toml: 1.0.0
✓ Versions are in sync

✓ Version bump complete!
ℹ Version updated to: 1.0.0

ℹ Next steps:
  1. Review the changes: git diff
  2. Run tests: cargo test --all
  3. Commit the changes: git add Cargo.toml pyproject.toml
  4. Create a commit: git commit -m "Bump version to 1.0.0"
  5. Create a tag: git tag v1.0.0
```

#### Best Practices

1. **Always test after bumping**: Run the full test suite to ensure nothing broke:
   ```bash
   ./scripts/bump-version.sh 1.0.0
   cargo test --all
   ```

2. **Use semantic versioning**: Follow [semver](https://semver.org/) conventions:
   - **MAJOR**: Breaking changes
   - **MINOR**: New features (backward compatible)
   - **PATCH**: Bug fixes (backward compatible)

3. **Dry run for important releases**: Preview changes first:
   ```bash
   ./scripts/bump-version.sh --dry-run 2.0.0
   ```

4. **Tag releases**: Create git tags for all releases:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

5. **Review before committing**: Always check the diff:
   ```bash
   git diff Cargo.toml pyproject.toml
   ```

#### Version Numbering Guidelines

Following semantic versioning (semver):

**Format**: `MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]`

- **MAJOR** (1.0.0): Breaking changes that require users to modify their code
- **MINOR** (1.1.0): New features that are backward compatible
- **PATCH** (1.0.1): Bug fixes that are backward compatible
- **PRERELEASE** (1.0.0-beta.1): Pre-release identifiers (alpha, beta, rc)
- **BUILD** (1.0.0+abc123): Build metadata

Examples:
- `1.0.0` - First stable release
- `1.1.0` - Added new features
- `1.1.1` - Fixed bugs in 1.1.0
- `2.0.0` - Breaking changes from 1.x
- `2.0.0-beta.1` - First beta of version 2.0.0
- `2.0.0-rc.1` - Release candidate for 2.0.0

#### Future Enhancements

Potential improvements for this script:

- **Git integration**: Optionally create commit and tag automatically
- **Version increment**: Support `--major`, `--minor`, `--patch` flags to auto-increment
- **Lockfile update**: Automatically update Cargo.lock
- **Dependency check**: Verify all dependencies are compatible with new version
- **Rollback**: Ability to revert to previous version

#### Contributing

When modifying this script:

1. Test with various version formats (stable, pre-release, build metadata)
2. Ensure it works on both Linux and macOS
3. Update this README with any new features or changes
4. Follow the project's bash scripting conventions

---

### update-site.sh

Automates the process of building documentation with MkDocs and generating the static site files.

#### Purpose

This script streamlines the workflow for building documentation by:

1. Validating that MkDocs is installed and configured
2. Building the documentation from `docs/` using MkDocs
3. Generating static site files into the `site/` directory

#### Prerequisites

Before using this script, ensure you have:

- **MkDocs installed**: Either via uv (recommended) or system-wide
  ```bash
  # Via uv (recommended)
  uv sync

  # Or system-wide
  pip install mkdocs mkdocs-material
  ```

- **Python 3.11+**: Required for MkDocs and dependencies (see `pyproject.toml`)

#### Usage

##### Basic Usage

Build documentation and generate the site:

```bash
./scripts/update-site.sh
```

This will build the documentation from `docs/` and output to `site/`.

##### Verbose Output

See detailed information about the build process:

```bash
./scripts/update-site.sh --verbose
```

Verbose mode shows:
- Full paths to directories
- MkDocs command being executed
- Build output from MkDocs

##### Build Without Cleaning

By default, the script runs `mkdocs build --clean` which removes the site directory before building. To skip cleaning:

```bash
./scripts/update-site.sh --no-clean
```

This is useful when:
- You want to preserve certain files in the site directory
- You're doing incremental builds during development

#### Command-Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Show help message and exit |
| `--verbose` | `-v` | Show detailed output for each step |
| `--no-clean` | | Build without cleaning the site directory first |

#### What the Script Does

1. **Validation Checks**
   - Verifies the script is run from the repository root
   - Checks that the `docs/` directory exists
   - Verifies MkDocs is available (via uv or system)

2. **Build Documentation**
   - Runs `mkdocs build` to generate the static site
   - Outputs the result to the `site/` directory
   - Shows build errors if any occur
   - Optionally cleans the site directory before building

#### Workflow Integration

##### Local Development

When working on documentation:

```bash
# Edit documentation in docs/
vim docs/guide/getting-started.md

# Preview changes locally
mkdocs serve

# When satisfied, build the site
./scripts/update-site.sh

# The site/ directory now contains the updated static files
# Commit and push as needed with git
```

##### CI/CD Integration

The script can be integrated into CI/CD pipelines for automated builds. Example for Woodpecker CI (`.woodpecker.yml`):

```yaml
steps:
  - name: build-site
    image: python:3.11
    commands:
      - pip install uv
      - uv sync
      - ./scripts/update-site.sh --verbose
    when:
      branch: main
      event: push
```

#### Troubleshooting

##### "MkDocs not found"

**Problem**: MkDocs is not installed or not in PATH.

**Solution**: Install MkDocs via uv or system-wide:
```bash
# Via uv (recommended)
uv sync

# Or system-wide
pip install mkdocs mkdocs-material
```

##### "Documentation directory not found"

**Problem**: The `docs/` directory doesn't exist.

**Solution**: Ensure you're running the script from the repository root and that the docs directory exists:
```bash
cd /path/to/wok
./scripts/update-site.sh
```

##### "Permission denied"

**Problem**: The script doesn't have execute permissions.

**Solution**: Make the script executable:
```bash
chmod +x scripts/update-site.sh
```

##### Build Errors

**Problem**: MkDocs encounters errors during the build.

**Solution**: Run with verbose output to see the full error:
```bash
./scripts/update-site.sh --verbose
```

Common issues:
- Missing dependencies in `mkdocs.yml`
- Broken links in markdown files
- Invalid YAML in `mkdocs.yml`
- Missing files referenced in navigation

#### Best Practices

1. **Always test locally first**: Use `mkdocs serve` to preview changes before building

2. **Use verbose for debugging**: When troubleshooting, run with `--verbose`:
   ```bash
   ./scripts/update-site.sh --verbose
   ```

3. **Commit generated files as needed**: The script only builds the site; you handle git operations:
   ```bash
   cd site
   git add -A
   git commit -m "Update site"
   git push
   cd ..
   ```

4. **Preview before deploying**: Check the generated files before pushing:
   ```bash
   ./scripts/update-site.sh
   cd site
   python -m http.server 8000  # Preview at http://localhost:8000
   ```

#### Script Output

The script provides color-coded output for easy reading:

- ✓ **Green**: Successful operations
- ✗ **Red**: Errors (will exit)
- ⚠ **Yellow**: Warnings
- ℹ **Blue**: Informational messages

Example output:

```
==> Checking prerequisites
✓ Running from repository root
✓ Documentation directory found

==> Building documentation with MkDocs
✓ Documentation built successfully
ℹ Output directory: /home/user/wok/site

✓ Site build complete!
ℹ The documentation site has been generated in: /home/user/wok/site
```

#### Future Enhancements

Potential improvements for this script:

- **Incremental builds**: Detect if docs have changed since last build
- **Link checking**: Validate internal and external links
- **Size reporting**: Report the size of the generated site
- **Performance metrics**: Show build time and statistics
- **Custom output directory**: Allow specifying alternate output location
- **Watch mode**: Rebuild on file changes (like `mkdocs serve` but for static builds)

#### Contributing

When modifying this script:

1. Test the build process thoroughly
2. Ensure error handling is robust
3. Update this README with any new features or changes
4. Follow the project's bash scripting conventions

#### License

This script is part of the Wok project and shares the same license.

## Adding New Scripts

When adding new scripts to this directory:

1. **Create the script file**: Use a `.sh` extension for shell scripts
2. **Add shebang**: Start with `#!/usr/bin/env bash`
3. **Set execute permissions**: `chmod +x scripts/your-script.sh`
4. **Document in this README**: Add a section describing the script
5. **Include help text**: Add a `--help` option to the script
6. **Follow conventions**: Use the same style as existing scripts (colors, error handling, etc.)
7. **Test thoroughly**: Test all options and edge cases

## Maintenance

Scripts in this directory should be:

- **Self-contained**: Minimize external dependencies
- **Portable**: Work on Linux, macOS, and WSL
- **Well-documented**: Include inline comments and README entries
- **Error-resistant**: Handle edge cases gracefully
- **User-friendly**: Provide clear output and helpful error messages
