# Scripts Directory

This directory contains utility scripts for maintaining the Wok project.

## Available Scripts

### update-site.sh

Automates the process of building documentation with MkDocs and generating the static site files.

#### Purpose

This script streamlines the workflow for building documentation by:

1. Validating that MkDocs is installed and configured
2. Building the documentation from `docs/` using MkDocs
3. Generating static site files into the `site/` directory

#### Prerequisites

Before using this script, ensure you have:

- **MkDocs installed**: Either via Poetry (recommended) or system-wide
  ```bash
  # Via Poetry (recommended)
  poetry install

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
   - Verifies MkDocs is available (via Poetry or system)

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
      - poetry install
      - ./scripts/update-site.sh --verbose
    when:
      branch: main
      event: push
```

#### Troubleshooting

##### "MkDocs not found"

**Problem**: MkDocs is not installed or not in PATH.

**Solution**: Install MkDocs via Poetry or system-wide:
```bash
# Via Poetry (recommended)
poetry install

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
