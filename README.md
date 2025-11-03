# Git Wok

[![status-badge](https://ci.codeberg.org/api/badges/12553/status.svg)](https://ci.codeberg.org/repos/12553)

**Git Wok** helps to organize multiple git repositories into a single multi-project workspace.

## 🚀 Beta Release

**Production Ready!** All core functionality is implemented, tested, and documented.

### ✨ Features

- **9 Complete Commands** for comprehensive multi-repo management
- **Housekeeping**: `init`, `status`
- **Package Management**: `add`, `remove`, `update`, `lock`
- **Development Flow**: `switch`, `push`
- **Release Management**: `tag`
- **Advanced Options**: Selective repo targeting, branch creation, GPG signing
- **Config Controls**: Per-repo skip lists to exclude projects from bulk operations without losing explicit access
- **Comprehensive Testing**: 37 tests covering all functionality

### 🎯 Quick Start

```bash
# Initialize a workspace
git-wok init

# Add a submodule
git-wok add path/to/submodule

# Switch all repos to current branch
git-wok switch --all

# Push changes to all repos
git-wok push --all

# Create and push a signed tag
git-wok tag --all v1.0.0 --sign --push
```

### 📚 Documentation

- [Getting Started Guide](docs/getting-started.md)
- [CLI Reference](docs/cli.md)
- [Configuration Guide](docs/wokfile.md)

## Community

- Join the Delta Chat group: [Git Wok on Delta Chat](https://i.delta.chat/#667BD2FB6B122F4138F29A17861B4E257DCDFDB9&a=lig%40countzero.co&g=Git%20Wok&x=0FgEK_cMRZ6NMvG1PAekdJE3&i=9Jn9KZM9tErF-O0k8xvadsn_&s=DyV77Vq3p4y86HX9rRuOMvm2)
