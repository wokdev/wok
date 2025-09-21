# Wok Roadmap

This document tracks the implementation status and future plans for Wok CLI.

## 🎯 Current Status: MVP Complete ✅

**Version 1.0.0-alpha** - All core functionality implemented and tested.

The Minimum Viable Product (MVP) provides complete functionality for managing multiple git repositories as a single workspace.

## 📋 Command Implementation Status

### ✅ Housekeeping Commands

| Command | Status | Description |
|---------|--------|-------------|
| `init` | ✅ **Implemented** | Create `wok.toml` and introspect existing submodules |
| `status` | ✅ **Implemented** | Show subprojects status (clean/dirty, branch info) |

### ✅ Package Management Commands

| Command | Status | Description |
|---------|--------|-------------|
| `add` | ✅ **Implemented** | Add subproject to config and as submodule |
| `remove` | ✅ **Implemented** | Remove subproject from config and submodule |
| `update` | ✅ **Implemented** | Switch repos to configured branch, fetch changes, merge |
| `lock` | ✅ **Implemented** | Ensure repos are on configured branch, commit submodule state |

### ✅ Development Flow Commands

| Command | Status | Description |
|---------|--------|-------------|
| `head switch` | ✅ **Implemented** | Switch all subrepos to umbrella's head branch |
| `switch` | ✅ **Implemented** | Switch specific repos with options (`--create`, `--all`, `--branch`) |
| `push` | ✅ **Implemented** | Push changes from configured repos to remotes |

### ✅ Release Flow Commands

| Command | Status | Description |
|---------|--------|-------------|
| `tag` | ✅ **Implemented** | Add tags to repos, show existing tags, sign and push |

## 📊 Progress Summary

- **Total Commands Planned**: 9
- **Implemented**: 9 (100%) ✅
- **MVP Status**: Complete ✅
- **Test Coverage**: 37 tests passing ✅

## 🚀 Future Development

### Phase 1: Technical Debt & Improvements (High Priority)

#### High Priority
- [ ] **Improve remote detection** (`src/repo.rs:208`)
  - Currently hardcoded to use "origin" remote
  - Should detect actual upstream remote for each branch

#### Medium Priority
- [ ] **Enhanced error handling**
  - Improve error messages for network operations
  - Add retry logic for fetch operations
  - Better conflict resolution guidance

- [ ] **Performance optimizations**
  - Parallel operations where possible
  - Caching for repeated operations
  - Progress indicators for long-running operations

### Phase 2: Advanced Features (Future Releases)

#### Potential New Commands
- [ ] **`diff`** - Show differences across repositories
- [ ] **`log`** - Unified log view across repositories
- [ ] **`clean`** - Clean untracked files across repositories
- [ ] **`stash`** - Stash changes across repositories

#### Advanced Features
- [ ] **Workspace templates** - Predefined workspace configurations
- [ ] **Plugin system** - Extensible command system
- [ ] **CI/CD integration** - Integration with popular CI/CD systems
- [ ] **Advanced filtering** - More sophisticated repo selection

## 🎯 Release Milestones

### ✅ Alpha Release (1.0.0-alpha) - Current
- All MVP commands implemented
- Comprehensive test coverage
- Basic documentation complete

### 🔄 Beta Release (1.0.0-beta) - Planned
- Address technical debt
- Performance improvements
- Enhanced error handling
- Real-world testing feedback incorporated

### 🎯 Stable Release (1.0.0) - Future
- Production-ready stability
- Complete documentation
- Performance optimizations
- Advanced features

## 📝 Development Notes

- The `status` command is implemented and exposed in the CLI interface
- Remote fetching and merging were recently implemented
- All commands have comprehensive test coverage
- The codebase uses git2 for git operations
- CLI is built with clap for argument parsing
- All commands support selective repository targeting

---

*Last updated: January 21, 2025*
*Implementation status based on current codebase analysis*
