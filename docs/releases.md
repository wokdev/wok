# Release Process

This document describes the process for creating a new release of Git Wok.

## Automated Release Pipeline

Git Wok uses Woodpecker CI to automatically build and publish releases when a tag is pushed.

### Supported Platforms

The CI pipeline builds binaries for:

- **Linux x86_64** (statically linked with musl)
- **Linux ARM64** (statically linked with musl)
- **macOS x86_64** (Intel)
- **macOS ARM64** (Apple Silicon)
- **Windows x86_64**

### Release Process

1. **Update Version**
   ```bash
   ./scripts/bump-version.sh X.Y.Z
   ```

2. **Review Changes**
   ```bash
   git diff Cargo.toml pyproject.toml
   ```

3. **Run Tests**
   ```bash
   cargo test --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   ```

4. **Commit Version Bump**
   ```bash
   git add Cargo.toml pyproject.toml Cargo.lock
   git commit -m "Bump version to X.Y.Z"
   ```

5. **Create and Push Tag**
   ```bash
   git tag vX.Y.Z
   git push origin main
   git push origin vX.Y.Z
   ```

6. **Monitor CI Pipeline**
   - Go to https://ci.codeberg.org/repos/12553
   - Watch the release pipeline execute
   - Verify all build steps succeed

7. **Verify Release**
   - Check https://codeberg.org/wok/wok/releases
   - Verify all binary artifacts are present
   - Download and test at least one binary

### Pre-release Versions

For beta, RC, or alpha releases:

```bash
./scripts/bump-version.sh X.Y.Z-beta.N
git add Cargo.toml pyproject.toml Cargo.lock
git commit -m "Bump version to X.Y.Z-beta.N"
git tag vX.Y.Z-beta.N
git push origin main --tags
```

### Troubleshooting

#### CI Pipeline Fails

1. Check the Woodpecker CI logs
2. Common issues:
   - Build errors: Fix code and push new commit
   - Cross-compilation issues: May need to update toolchain versions
   - Token expired: Regenerate `codeberg_token` secret

#### Release Already Exists

If you need to republish a release:

1. Delete the existing release on Codeberg
2. Delete the tag: `git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`
3. Re-create and push the tag

#### Manual Release

If the CI pipeline is unavailable:

1. Build locally for your platform:
   ```bash
   cargo build --release
   ```

2. Create release manually on Codeberg

3. Upload binary as `wok-<platform>-<arch>`

## CI Configuration

### Required Secrets

The release pipeline requires a Codeberg API token:

- **Name**: `codeberg_token`
- **Scopes**: `write:repository`
- **Location**: Repository Settings ? Secrets

### Pipeline Files

- `.woodpecker/release.yml` - Main release pipeline (triggers on tag push)
- `.woodpecker/test.yml` - Test pipeline (runs on push to src/tests)

### Updating the Pipeline

To modify the release process:

1. Edit `.woodpecker/release.yml`
2. Test changes on a test tag
3. Delete test tag and release
4. Commit pipeline changes

## Post-Release Tasks

After a successful release:

1. **Announce Release**
   - Post to Delta Chat group
   - Update documentation site if needed

2. **Verify Installation**
   - Test installation instructions from docs
   - Check download links work

3. **Update Dependencies** (if needed)
   - Run `cargo update`
   - Test and commit if appropriate

## Release Schedule

Git Wok follows semantic versioning:

- **Patch releases** (X.Y.Z): Bug fixes, typically as needed
- **Minor releases** (X.Y.0): New features, backward compatible
- **Major releases** (X.0.0): Breaking changes, planned in advance

No fixed release schedule; releases are made when features or fixes are ready.
