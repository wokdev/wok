# Git Wok Roadmap

## 🎯 Current Status: Beta Release

All core functionality complete, polished documentation, ready for production testing.

Git Wok provides complete functionality for managing multiple git repositories as a single workspace using git submodules.

## 📋 Available Commands

All 9 commands are fully implemented and tested:

| Category | Command | Description |
|----------|---------|-------------|
| **Workspace Setup** | `init` | Initialize workspace from existing submodules |
| | `assemble` | Assemble workspace from directory of repos |
| **Housekeeping** | `status` | Show workspace and repo status |
| **Repository Management** | `add` | Add submodule to configuration |
| | `rm` | Remove submodule from configuration |
| **Branch Management** | `switch` | Switch repos with fine-grained control (use `--all` for bulk operations) |
| **Synchronization** | `lock` | Commit current submodule state |
| | `update` | Fetch and merge from remotes |
| **Remote Operations** | `push` | Push changes to remotes |
| **Release Management** | `tag` | Create, sign, and push tags |
| **Utilities** | `completion` | Generate shell completions |

## ✨ Features

- **Complete Command Set**: All planned commands implemented
- **Selective Targeting**: Branch-based, all repos, or explicit repo selection
- **Skip Lists**: Per-repo exclusion from bulk operations
- **Branch Creation**: Create branches on-the-fly with `--create` flag
- **GPG Signing**: Sign tags with GPG
- **Shell Completion**: Bash, Zsh, and Fish support
- **Comprehensive Testing**: 37+ test cases covering all functionality
- **Complete Documentation**: CLI reference, getting started guide, and examples

## 🚀 Path to 1.0.0 Stable

### Beta Phase (Current)
- ✅ All core commands implemented
- ✅ Comprehensive documentation
- ✅ Full test coverage
- ✅ TOML configuration format
- 🔄 Real-world production testing
- 🔄 User feedback incorporation

### Stable Release Goals
- [ ] Address user feedback from beta testing
- [ ] Performance testing with large workspaces
- [ ] Enhanced error messages based on real-world usage
- [ ] Finalize configuration format as stable

## 💡 Future Enhancements

These features are under consideration for post-1.0 releases:

### Performance & Usability
- Parallel operations for better performance
- Progress indicators for long-running operations
- Enhanced remote detection (currently hardcoded to "origin")
- Caching for repeated operations

### Additional Commands (Maybe)
- `diff` - Show differences across repositories
- `log` - Unified log view across repositories
- `clean` - Clean untracked files across repositories
- `stash` - Stash changes across repositories

### Advanced Features (Maybe)
- Workspace templates for common configurations
- Integration with CI/CD systems
- Advanced filtering and repo selection patterns
- Plugin/extension system

## 🤝 Contributing

Git Wok is open to contributions! If you encounter bugs or have feature requests:

1. Check existing issues on Codeberg
2. Open a new issue with clear description
3. Join the discussion in [#wok:matrix.org](https://matrix.to/#/#wok:matrix.org)

## 📈 Release History

- **Beta releases** (2025) - Documentation, refinements, production testing
- **Alpha release** (Jan 2025) - MVP completion, all commands implemented

---

*Last updated: October 21, 2025*
