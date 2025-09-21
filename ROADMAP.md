# Wok Roadmap

This document tracks the implementation status of Wok CLI commands and features.

## 🎯 MVP Goals

The Minimum Viable Product (MVP) aims to provide core functionality for managing multiple git repositories as a single workspace.

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

### ⚠️ Development Flow Commands

| Command | Status | Description | Priority |
|---------|--------|-------------|----------|
| `head switch` | ✅ **Implemented** | Switch all subrepos to umbrella's head branch | ✅ |
| `switch` | ❌ **Missing** | Switch specific repos with options (`--create`, `--all`, `--branch`) | 🔥 **High** |
| `push` | ❌ **Missing** | Push changes from configured repos to remotes | 🔥 **High** |

### ❌ Release Flow Commands

| Command | Status | Description | Priority |
|---------|--------|-------------|----------|
| `tag` | ❌ **Missing** | Add tags to repos, show existing tags, sign and push | 🟡 **Medium** |

## 🚀 Next Steps for MVP

### Phase 1: Core Development Flow (High Priority)
- [ ] **Implement `switch` command**
  - [ ] Add CLI argument parsing for `--create`, `--all`, `--branch` options
  - [ ] Implement selective repo targeting
  - [ ] Add branch creation logic
  - [ ] Integrate with existing `lock` functionality
  - [ ] Add comprehensive tests

- [ ] **Implement `push` command**
  - [ ] Add CLI argument parsing for `-u/--set-upstream`, `--all`, `--branch` options
  - [ ] Implement git push operations using git2
  - [ ] Add upstream configuration logic
  - [ ] Implement selective repo targeting
  - [ ] Add comprehensive tests

### Phase 2: Release Management (Medium Priority)
- [ ] **Implement `tag` command**
  - [ ] Add CLI argument parsing for complex options
  - [ ] Implement tag creation and management
  - [ ] Add tag signing support
  - [ ] Implement tag pushing functionality
  - [ ] Add comprehensive tests

## 🔧 Technical Debt & Improvements

### High Priority
- [ ] **Improve remote detection** (`src/repo.rs:208`)
  - Currently hardcoded to use "origin" remote
  - Should detect actual upstream remote for each branch

### Medium Priority
- [ ] **Expose `status` command in CLI**
  - Command is implemented but not accessible via CLI
  - Add to `src/bin/wok.rs` command structure

- [ ] **Enhanced error handling**
  - Improve error messages for network operations
  - Add retry logic for fetch operations
  - Better conflict resolution guidance

### Low Priority
- [ ] **Performance optimizations**
  - Parallel operations where possible
  - Caching for repeated operations
  - Progress indicators for long-running operations

## 📊 Progress Summary

- **Total Commands Planned**: 8
- **Implemented**: 6 (75%)
- **Missing for MVP**: 2 (25%)
- **Missing Overall**: 2 (25%)

## 🎯 MVP Completion Criteria

The MVP will be considered complete when:
- [ ] All core package management commands work reliably
- [ ] Development flow commands (`switch`, `push`) are implemented
- [ ] All commands have comprehensive test coverage
- [ ] Documentation is complete and accurate
- [ ] No critical bugs or security issues

## 📝 Notes

- The `status` command is implemented but not exposed in the CLI interface
- Remote fetching and merging were recently implemented (commit e8c22a6)
- All existing commands have test coverage
- The codebase uses git2 for git operations
- CLI is built with clap for argument parsing

---

*Last updated: $(date)*
*Implementation status based on current codebase analysis*
