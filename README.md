# Wok

[![status-badge](https://ci.codeberg.org/api/badges/12553/status.svg)](https://ci.codeberg.org/repos/12553)

**Wok** helps to organize multiple git repositories into a single multi-project workspace.

## 🚀 Version 1.0.0-alpha

**MVP Complete!** All core functionality is implemented and tested.

### ✨ Features

- **9 Complete Commands** for comprehensive multi-repo management
- **Housekeeping**: `init`, `status`
- **Package Management**: `add`, `remove`, `update`, `lock`
- **Development Flow**: `head switch`, `switch`, `push`
- **Release Management**: `tag`
- **Advanced Options**: Selective repo targeting, branch creation, GPG signing
- **Comprehensive Testing**: 37 tests covering all functionality

### 🎯 Quick Start

```bash
# Initialize a workspace
wok init

# Add a submodule
wok repo add path/to/submodule

# Switch all repos to current branch
wok head switch

# Push changes to all repos
wok push --all

# Create and push a signed tag
wok tag --create v1.0.0 --sign --push --all
```

### 📚 Documentation

- [Getting Started Guide](docs/getting-started.md)
- [CLI Reference](docs/cli.md)
- [Configuration Guide](docs/wokfile.md)
- [Changelog](CHANGELOG.md)

## Community

Meet us in the chat room: [#wok:matrix.org](https://matrix.to/#/#wok:matrix.org)
