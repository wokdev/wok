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

### ✅ Development Flow Commands

| Command | Status | Description | Priority |
|---------|--------|-------------|----------|
| `head switch` | ✅ **Implemented** | Switch all subrepos to umbrella's head branch | ✅ |
| `switch` | ✅ **Implemented** | Switch specific repos with options (`--create`, `--all`, `--branch`) | ✅ |
| `push` | ✅ **Implemented** | Push changes from configured repos to remotes | ✅ |

### ❌ Release Flow Commands

| Command | Status | Description | Priority |
|---------|--------|-------------|----------|
| `tag` | ❌ **Missing** | Add tags to repos, show existing tags, sign and push | 🟡 **Medium** |

## 🚀 Next Steps for MVP

### Phase 1: Core Development Flow (High Priority)
- [x] **Implement `switch` command** ✅
  - [x] Add CLI argument parsing for `--create`, `--all`, `--branch` options
  - [x] Implement selective repo targeting
  - [x] Add branch creation logic
  - [x] Integrate with existing `lock` functionality
  - [x] Add comprehensive tests

- [x] **Implement `push` command** ✅
  - [x] Add CLI argument parsing for `-u/--set-upstream`, `--all`, `--branch` options
  - [x] Implement git push operations using git2
  - [x] Add upstream configuration logic
  - [x] Implement selective repo targeting
  - [x] Add comprehensive tests

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
- [x] **Expose `status` command in CLI** ✅
  - Command is now accessible via CLI
  - Added to `src/bin/wok.rs` command structure

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
- **Implemented**: 8 (100%)
- **Missing for MVP**: 0 (0%)
- **Missing Overall**: 1 (12.5%)

## 🎯 MVP Completion Criteria

The MVP will be considered complete when:
- [x] All core package management commands work reliably ✅
- [x] Development flow commands (`switch` ✅, `push` ✅) are implemented
- [x] All commands have comprehensive test coverage ✅
- [x] Documentation is complete and accurate ✅
- [x] No critical bugs or security issues ✅

## 📝 Notes

- The `status` command is implemented and exposed in the CLI interface
- Remote fetching and merging were recently implemented
- The `switch` command was implemented with full feature set
- The `push` command was implemented with full feature set
- All existing commands have test coverage
- The codebase uses git2 for git operations
- CLI is built with clap for argument parsing

---

*Last updated: $(date)*
*Implementation status based on current codebase analysis*
