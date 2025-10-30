# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Support for git rebase strategy during `update` operations, respecting user's pull strategy preferences configured via `pull.rebase` and `branch.<name>.rebase` settings

### Changed
- `update` command now respects git's `pull.rebase` and `branch.<name>.rebase` configuration settings. Repositories configured to use rebase will have their changes rebased instead of merged when updating

## [1.0.0-alpha] - 2025-01-21

### Added
- **Complete CLI implementation** with all 9 planned commands
- **Housekeeping Commands:**
  - `init` - Create `wok.toml` and introspect existing submodules
  - `status` - Show subprojects status (clean/dirty, branch info)
- **Package Management Commands:**
  - `add` - Add subproject to config and as submodule
  - `remove` - Remove subproject from config and submodule
  - `update` - Switch repos to configured branch, fetch changes, merge
  - `lock` - Ensure repos are on configured branch, commit submodule state
- **Development Flow Commands:**
  - `head switch` - Switch all subrepos to umbrella's head branch
  - `switch` - Switch specific repos with options (`--create`, `--all`, `--branch`)
  - `push` - Push changes from configured repos to remotes
- **Release Flow Commands:**
  - `tag` - Add tags to repos, show existing tags, sign and push
- **Advanced Features:**
  - Selective repository targeting (all repos, specific repos, branch-based)
  - Branch creation with `--create` option
  - Upstream configuration for new branches
  - GPG tag signing support
  - Tag pushing to remote repositories
  - Comprehensive error handling and user feedback
- **Testing:**
  - Complete test coverage with 37 test cases
  - Tests for all commands and edge cases
  - Integration tests for CLI functionality
- **Documentation:**
  - Comprehensive CLI help system
  - Detailed command documentation
  - Getting started guide
  - API documentation

### Technical Details
- Built with Rust using git2 library for robust git operations
- CLI built with clap for argument parsing
- Comprehensive error handling with anyhow
- Support for complex git workflows and operations
- Cross-platform compatibility

### MVP Completion
This alpha release represents the completion of the Minimum Viable Product (MVP) with all core functionality implemented and tested. The tool provides a complete solution for managing multiple git repositories as a single workspace.

---

## Release Notes

### What's Working
- All 9 planned commands are fully functional
- Comprehensive test coverage (37 tests passing)
- Robust error handling and user feedback
- Support for complex git workflows
- Cross-platform compatibility

### Known Limitations
- Remote detection is currently hardcoded to "origin" (see Technical Debt section)
- Some advanced git features may need additional testing in real-world scenarios
- Performance optimizations for large repositories are planned for future releases

### Recommended Pre-Release Testing
1. **Real-world testing** with actual multi-repo projects
2. **Performance testing** with large repositories
3. **Cross-platform testing** on different operating systems
4. **Integration testing** with various git hosting services
5. **User acceptance testing** with target audience

### Next Steps for Beta Release
1. Address technical debt items (remote detection, performance optimizations)
2. Implement additional error handling improvements
3. Add progress indicators for long-running operations
4. Consider parallel operations for better performance
5. Enhanced documentation and examples
