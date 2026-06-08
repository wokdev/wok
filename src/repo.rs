use std::{fmt, fs, io::ErrorKind, path, process::Command};

use anyhow::*;
use git2::StatusOptions;
use git2::build::CheckoutBuilder;
use std::result::Result::Ok;

#[derive(Debug, Clone, PartialEq)]
pub enum MergeResult {
    UpToDate,
    FastForward,
    Merged,
    Rebased,
    Conflicts,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RemoteComparison {
    UpToDate,
    Ahead(usize),
    Behind(usize),
    Diverged(usize, usize),
    NoRemote,
}

pub struct Repo {
    pub git_repo: git2::Repository,
    pub work_dir: path::PathBuf,
    pub head: String,
    pub subrepos: Vec<Repo>,
}

pub fn init_configured_submodules(
    work_dir: &path::Path,
    configured_paths: &[path::PathBuf],
) -> Result<()> {
    let git_repo = git2::Repository::open(work_dir)
        .with_context(|| format!("Cannot open repo at `{}`", work_dir.display()))?;

    // Initialize shallower paths first so nested configured paths can be
    // initialized through already materialized parent worktrees.
    let mut sorted_paths = configured_paths.to_vec();
    sorted_paths.sort_by_key(|path| path.components().count());

    for configured_path in &sorted_paths {
        init_configured_submodule_path(&git_repo, work_dir, configured_path)
            .with_context(|| {
                format!(
                    "Cannot initialize configured submodule `{}`",
                    configured_path.display()
                )
            })?;
    }

    Ok(())
}

fn init_configured_submodule_path(
    git_repo: &git2::Repository,
    work_dir: &path::Path,
    configured_path: &path::Path,
) -> Result<()> {
    if configured_path.as_os_str().is_empty() {
        return Ok(());
    }

    let mut components = configured_path.components();
    let first_component = match components.next() {
        Some(component) => component.as_os_str(),
        None => return Ok(()),
    };

    let first_component_str = first_component.to_str().with_context(|| {
        format!(
            "Configured submodule path `{}` is not valid UTF-8",
            configured_path.display()
        )
    })?;

    let mut submodule = match git_repo.find_submodule(first_component_str) {
        Ok(submodule) => submodule,
        Err(err) if err.code() == git2::ErrorCode::NotFound => return Ok(()),
        Err(err) => return Err(err.into()),
    };

    submodule.init(false).with_context(|| {
        format!(
            "Cannot initialize submodule `{}` in `{}`",
            first_component_str,
            work_dir.display()
        )
    })?;

    let submodule_work_dir = work_dir.join(first_component);
    let is_initialized = submodule.open().is_ok();
    if !is_initialized {
        let module_git_dir = git_repo.path().join("modules").join(first_component);
        if module_git_dir.exists() && !submodule_work_dir.exists() {
            fs::create_dir_all(&submodule_work_dir).with_context(|| {
                format!(
                    "Cannot create submodule worktree directory `{}`",
                    submodule_work_dir.display()
                )
            })?;
        }

        if let Err(initial_err) = submodule.update(false, None) {
            submodule.update(true, None).with_context(|| {
                format!(
                    "Cannot update submodule `{}` in `{}` (initial attempt with init=false failed: {})",
                    first_component_str,
                    work_dir.display(),
                    initial_err,
                )
            })?;
        }
    }

    let remaining_path = components.as_path();
    if remaining_path.as_os_str().is_empty() {
        return Ok(());
    }

    let child_work_dir = submodule_work_dir;
    let child_repo = git2::Repository::open(&child_work_dir).with_context(|| {
        format!(
            "Cannot open initialized submodule repo at `{}`",
            child_work_dir.display()
        )
    })?;

    init_configured_submodule_path(&child_repo, &child_work_dir, remaining_path)
}

impl Repo {
    pub fn new(work_dir: &path::Path, head_name: Option<&str>) -> Result<Self> {
        let git_repo = git2::Repository::open(work_dir)
            .with_context(|| format!("Cannot open repo at `{}`", work_dir.display()))?;

        let head = match head_name {
            Some(name) => String::from(name),
            None => {
                let is_detached = git_repo.head_detached().with_context(|| {
                    format!(
                        "Cannot determine head state for repo at `{}`",
                        work_dir.display()
                    )
                })?;
                if is_detached {
                    String::from("<detached>")
                } else {
                    String::from(git_repo.head().with_context(|| {
                        format!(
                            "Cannot find the head branch for repo at `{}`. Is it detached?",
                            work_dir.display()
                        )
                    })?.shorthand().with_context(|| {
                        format!(
                            "Cannot find a human readable representation of the head ref for repo at `{}`",
                            work_dir.display(),
                        )
                    })?)
                }
            },
        };

        let subrepos = git_repo
            .submodules()
            .with_context(|| {
                format!(
                    "Cannot load submodules for repo at `{}`",
                    work_dir.display()
                )
            })?
            .iter()
            .map(|submodule| Repo::new(&work_dir.join(submodule.path()), None))
            .collect::<Result<Vec<Repo>>>()?;

        Ok(Repo {
            git_repo,
            work_dir: path::PathBuf::from(work_dir),
            head,
            subrepos,
        })
    }

    pub fn get_subrepo_by_path(&self, subrepo_path: &path::PathBuf) -> Option<&Repo> {
        self.subrepos
            .iter()
            .find(|subrepo| subrepo.work_dir == self.work_dir.join(subrepo_path))
    }

    pub fn sync(&self) -> Result<()> {
        self.switch(&self.head)?;
        Ok(())
    }

    pub fn uses_lfs(&self) -> Result<bool> {
        let attributes_path = self.work_dir.join(".gitattributes");
        if attributes_path.exists() {
            let attributes =
                fs::read_to_string(&attributes_path).with_context(|| {
                    format!("Cannot read `{}`", attributes_path.display())
                })?;
            if attributes.lines().any(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty()
                    && !trimmed.starts_with('#')
                    && trimmed.contains("filter=lfs")
            }) {
                return Ok(true);
            }
        }

        if self.work_dir.join(".lfsconfig").exists() {
            return Ok(true);
        }

        // Repository::path() points to the actual git directory, including for
        // submodules/worktrees where `.git` in the work tree is a redirect file.
        if self.git_repo.path().join("lfs").exists() {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn lfs_pull_if_needed(&self) -> Result<()> {
        if self.uses_lfs()? {
            self.run_git_lfs(&["pull"])?;
        }
        Ok(())
    }

    pub fn lfs_push_if_needed(
        &self,
        remote_name: &str,
        branch_name: &str,
    ) -> Result<()> {
        if self.uses_lfs()? {
            self.run_git_lfs(&["push", remote_name, branch_name])?;
        }
        Ok(())
    }

    fn run_git_lfs(&self, args: &[&str]) -> Result<()> {
        let output = Command::new("git-lfs")
            .args(args)
            .current_dir(&self.work_dir)
            .output()
            .map_err(|err| {
                if err.kind() == ErrorKind::NotFound {
                    anyhow!(
                        "Git LFS support required for `{}` but `git-lfs` is not installed.\n\
                        Install it on Fedora with:\n\
                        sudo dnf install git-lfs\n\
                        git lfs install",
                        self.work_dir.display()
                    )
                } else {
                    anyhow!(err).context(format!(
                        "Cannot execute git-lfs in `{}`",
                        self.work_dir.display()
                    ))
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let details = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                "no output".to_string()
            };
            bail!(
                "git-lfs {} failed in `{}`: {}",
                args.join(" "),
                self.work_dir.display(),
                details
            );
        }

        Ok(())
    }

    pub fn switch(&self, head: &str) -> Result<()> {
        self.git_repo.set_head(&self.resolve_reference(head)?)?;
        let checkout_result = self.git_repo.checkout_head(None);
        checkout_result?;
        Ok(())
    }

    pub fn switch_force(&self, head: &str) -> Result<()> {
        self.git_repo.set_head(&self.resolve_reference(head)?)?;
        let mut checkout = CheckoutBuilder::new();
        checkout.force();
        let checkout_result = self.git_repo.checkout_head(Some(&mut checkout));
        checkout_result?;
        Ok(())
    }

    pub fn refresh_worktree(&self) -> Result<()> {
        let checkout_result = self.git_repo.checkout_head(None);
        checkout_result?;
        Ok(())
    }

    pub fn refresh_worktree_force(&self) -> Result<()> {
        let mut checkout = CheckoutBuilder::new();
        checkout.force();
        let checkout_result = self.git_repo.checkout_head(Some(&mut checkout));
        checkout_result?;
        Ok(())
    }

    pub fn checkout_path_from_head(&self, path: &path::Path) -> Result<()> {
        let mut checkout = CheckoutBuilder::new();
        checkout.force().path(path);
        self.git_repo.checkout_head(Some(&mut checkout))?;
        Ok(())
    }

    fn switch_forced(&self, head: &str) -> Result<()> {
        self.git_repo.set_head(&self.resolve_reference(head)?)?;
        let mut checkout = CheckoutBuilder::new();
        checkout.force();
        self.git_repo.checkout_head(Some(&mut checkout))?;
        Ok(())
    }

    pub fn fetch(&self) -> Result<()> {
        if self.git_repo.head_detached().with_context(|| {
            format!(
                "Cannot determine head state for repo at `{}`",
                self.work_dir.display()
            )
        })? {
            return Ok(());
        }

        // Get the remote for the current branch
        let head_ref = self.git_repo.head()?;
        let branch_name = head_ref.shorthand().with_context(|| {
            format!(
                "Cannot get branch name for repo at `{}`",
                self.work_dir.display()
            )
        })?;

        let tracking = match self.tracking_branch(branch_name)? {
            Some(tracking) => tracking,
            None => {
                // No upstream configured, skip fetch
                return Ok(());
            },
        };

        // Check if remote exists
        match self.git_repo.find_remote(&tracking.remote) {
            Ok(mut remote) => {
                let mut fetch_options = git2::FetchOptions::new();
                fetch_options.remote_callbacks(self.remote_callbacks()?);

                remote
                    .fetch::<&str>(&[], Some(&mut fetch_options), None)
                    .with_context(|| {
                        format!(
                            "Failed to fetch from remote '{}' for repo at `{}`\n\
                            \n\
                            Possible causes:\n\
                            - SSH agent not running or not accessible (check SSH_AUTH_SOCK)\n\
                            - SSH keys not properly configured in ~/.ssh/\n\
                            - Credential helper not configured (git config credential.helper)\n\
                            - Network/firewall issues\n\
                            \n\
                            Try running: git fetch --verbose\n\
                            Or check authentication with: git-wok test-auth",
                            tracking.remote,
                            self.work_dir.display()
                        )
                    })?;
            },
            Err(_) => {
                // No remote configured, skip fetch
                return Ok(());
            },
        }

        Ok(())
    }

    pub fn ensure_on_branch(&self, branch_name: &str) -> Result<()> {
        if !self.is_worktree_clean()? {
            bail!(
                "Refusing to switch branches with uncommitted changes in `{}`",
                self.work_dir.display()
            );
        }

        if !self.git_repo.head_detached().with_context(|| {
            format!(
                "Cannot determine head state for repo at `{}`",
                self.work_dir.display()
            )
        })? && let Ok(head) = self.git_repo.head()
            && head.shorthand().ok() == Some(branch_name)
        {
            return Ok(());
        }

        let local_ref = format!("refs/heads/{}", branch_name);
        if self.git_repo.find_reference(&local_ref).is_ok() {
            self.switch_forced(branch_name)?;
            return Ok(());
        }

        let remote_name = self.get_remote_name_for_branch(branch_name)?;
        if let Ok(mut remote) = self.git_repo.find_remote(&remote_name) {
            let mut fetch_options = git2::FetchOptions::new();
            fetch_options.remote_callbacks(self.remote_callbacks()?);
            remote.fetch::<&str>(&[], Some(&mut fetch_options), None)?;
        }

        let remote_ref = format!("refs/remotes/{}/{}", remote_name, branch_name);
        if let Ok(remote_oid) = self.git_repo.refname_to_id(&remote_ref) {
            let remote_commit = self.git_repo.find_commit(remote_oid)?;
            self.git_repo.branch(branch_name, &remote_commit, false)?;
            let mut local_branch = self
                .git_repo
                .find_branch(branch_name, git2::BranchType::Local)?;
            local_branch
                .set_upstream(Some(&format!("{}/{}", remote_name, branch_name)))?;
            self.switch(branch_name)?;
            return Ok(());
        }

        let head = self.git_repo.head()?;
        let current_commit = head.peel_to_commit()?;
        self.git_repo.branch(branch_name, &current_commit, false)?;
        self.switch(branch_name)?;
        Ok(())
    }

    pub fn ensure_on_branch_existing_or_remote(
        &self,
        branch_name: &str,
        create: bool,
    ) -> Result<()> {
        if !self.is_worktree_clean()? {
            bail!(
                "Refusing to switch branches with uncommitted changes in `{}`",
                self.work_dir.display()
            );
        }

        if !self.git_repo.head_detached().with_context(|| {
            format!(
                "Cannot determine head state for repo at `{}`",
                self.work_dir.display()
            )
        })? && let Ok(head) = self.git_repo.head()
            && head.shorthand().ok() == Some(branch_name)
        {
            return Ok(());
        }

        let local_ref = format!("refs/heads/{}", branch_name);
        if self.git_repo.find_reference(&local_ref).is_ok() {
            self.switch(branch_name)?;
            return Ok(());
        }

        let remote_name = self.get_remote_name_for_branch(branch_name)?;
        if let Ok(mut remote) = self.git_repo.find_remote(&remote_name) {
            let mut fetch_options = git2::FetchOptions::new();
            fetch_options.remote_callbacks(self.remote_callbacks()?);
            remote.fetch::<&str>(&[], Some(&mut fetch_options), None)?;
        }

        let remote_ref = format!("refs/remotes/{}/{}", remote_name, branch_name);
        if let Ok(remote_oid) = self.git_repo.refname_to_id(&remote_ref) {
            let remote_commit = self.git_repo.find_commit(remote_oid)?;
            self.git_repo.branch(branch_name, &remote_commit, false)?;
            let mut local_branch = self
                .git_repo
                .find_branch(branch_name, git2::BranchType::Local)?;
            local_branch
                .set_upstream(Some(&format!("{}/{}", remote_name, branch_name)))?;
            self.switch_forced(branch_name)?;
            return Ok(());
        }

        if create {
            let head = self.git_repo.head()?;
            let current_commit = head.peel_to_commit()?;
            self.git_repo.branch(branch_name, &current_commit, false)?;
            self.switch_forced(branch_name)?;
            return Ok(());
        }

        bail!(
            "Branch '{}' does not exist and --create not specified",
            branch_name
        );
    }

    /// Switch the repo to `branch_name` without touching the working tree beyond
    /// what is strictly needed to update HEAD (no force-checkout, no clean check).
    /// If the branch does not exist locally and `create` is true, create it at the
    /// current HEAD commit. If `create` is false and the branch is not found
    /// locally, bail with an error. Remote tracking is not consulted.
    ///
    /// This is the right call when the caller wants to preserve uncommitted changes.
    pub fn switch_or_create_preserving(
        &self,
        branch_name: &str,
        create: bool,
    ) -> Result<()> {
        // Already on the target branch — nothing to do.
        if !self.git_repo.head_detached().with_context(|| {
            format!(
                "Cannot determine head state for repo at `{}`",
                self.work_dir.display()
            )
        })? && let Ok(head) = self.git_repo.head()
            && head.shorthand().ok() == Some(branch_name)
        {
            return Ok(());
        }

        let local_ref = format!("refs/heads/{}", branch_name);
        if self.git_repo.find_reference(&local_ref).is_ok() {
            // Branch exists locally: update HEAD without touching the worktree.
            self.git_repo.set_head(&local_ref)?;
            return Ok(());
        }

        if create {
            let head = self.git_repo.head()?;
            let current_commit = head.peel_to_commit()?;
            self.git_repo.branch(branch_name, &current_commit, false)?;
            self.git_repo.set_head(&local_ref)?;
            return Ok(());
        }

        bail!(
            "Branch '{}' does not exist and --create not specified",
            branch_name
        );
    }

    fn rebase(
        &self,
        _branch_name: &str,
        remote_commit: &git2::Commit,
    ) -> Result<MergeResult> {
        let _local_commit = self.git_repo.head()?.peel_to_commit()?;
        let remote_oid = remote_commit.id();

        // Prepare annotated commit for rebase
        let remote_annotated = self.git_repo.find_annotated_commit(remote_oid)?;

        // Initialize rebase operation
        let signature = self.git_repo.signature()?;
        let mut rebase = self.git_repo.rebase(
            None,                    // branch to rebase (None = HEAD)
            Some(&remote_annotated), // upstream
            None,                    // onto (None = upstream)
            None,                    // options
        )?;

        // Process each commit in the rebase
        let mut has_conflicts = false;
        while let Some(op) = rebase.next() {
            match op {
                Ok(_rebase_op) => {
                    // Check for conflicts
                    let index = self.git_repo.index()?;
                    if index.has_conflicts() {
                        has_conflicts = true;
                        break;
                    }

                    // Commit the rebased changes
                    if rebase.commit(None, &signature, None).is_err() {
                        has_conflicts = true;
                        break;
                    }
                },
                Err(_) => {
                    has_conflicts = true;
                    break;
                },
            }
        }

        if has_conflicts {
            // Leave repository in state with conflicts for user to resolve
            return Ok(MergeResult::Conflicts);
        }

        // Finish the rebase
        rebase.finish(Some(&signature))?;

        Ok(MergeResult::Rebased)
    }

    pub fn merge(&self, branch_name: &str) -> Result<MergeResult> {
        // First, fetch the latest changes
        self.fetch()?;

        // Resolve the tracking branch reference
        let tracking = match self.tracking_branch(branch_name)? {
            Some(tracking) => tracking,
            None => {
                // No upstream configured, treat as up to date
                return Ok(MergeResult::UpToDate);
            },
        };

        // Check if remote branch exists
        let remote_branch_oid = match self.git_repo.refname_to_id(&tracking.remote_ref)
        {
            Ok(oid) => oid,
            Err(_) => {
                // No remote branch, just return up to date
                return Ok(MergeResult::UpToDate);
            },
        };

        let remote_commit = self.git_repo.find_commit(remote_branch_oid)?;
        let local_commit = self.git_repo.head()?.peel_to_commit()?;

        // Check if we're already up to date
        if local_commit.id() == remote_commit.id() {
            return Ok(MergeResult::UpToDate);
        }

        // Check if we can fast-forward (works for both merge and rebase)
        if self
            .git_repo
            .graph_descendant_of(remote_commit.id(), local_commit.id())?
        {
            // Fast-forward merge
            self.git_repo.reference(
                &format!("refs/heads/{}", branch_name),
                remote_commit.id(),
                true,
                &format!("Fast-forward '{}' to {}", branch_name, tracking.remote_ref),
            )?;
            self.git_repo
                .set_head(&format!("refs/heads/{}", branch_name))?;
            let mut checkout = CheckoutBuilder::new();
            checkout.force();
            self.git_repo.checkout_head(Some(&mut checkout))?;
            self.lfs_pull_if_needed()?;
            return Ok(MergeResult::FastForward);
        }

        // Determine pull strategy from git config
        let pull_strategy = self.get_pull_strategy(branch_name)?;

        match pull_strategy {
            PullStrategy::Rebase => {
                // Perform rebase
                let result = self.rebase(branch_name, &remote_commit)?;
                if matches!(result, MergeResult::Rebased) {
                    self.lfs_pull_if_needed()?;
                }
                Ok(result)
            },
            PullStrategy::Merge => {
                // Perform merge (existing logic)
                let result = self.do_merge(
                    branch_name,
                    &local_commit,
                    &remote_commit,
                    &tracking,
                )?;
                if matches!(result, MergeResult::Merged) {
                    self.lfs_pull_if_needed()?;
                }
                Ok(result)
            },
        }
    }

    fn do_merge(
        &self,
        branch_name: &str,
        local_commit: &git2::Commit,
        remote_commit: &git2::Commit,
        tracking: &TrackingBranch,
    ) -> Result<MergeResult> {
        // Perform a merge
        let mut merge_opts = git2::MergeOptions::new();
        merge_opts.fail_on_conflict(false); // Don't fail on conflicts, we'll handle them

        let _merge_result = self.git_repo.merge_commits(
            local_commit,
            remote_commit,
            Some(&merge_opts),
        )?;

        // Check if there are conflicts by examining the index
        let mut index = self.git_repo.index()?;
        let has_conflicts = index.has_conflicts();

        if !has_conflicts {
            // No conflicts, merge was successful
            let signature = self.git_repo.signature()?;
            let tree_id = index.write_tree()?;
            let tree = self.git_repo.find_tree(tree_id)?;

            self.git_repo.commit(
                Some(&format!("refs/heads/{}", branch_name)),
                &signature,
                &signature,
                &format!("Merge remote-tracking branch '{}'", tracking.remote_ref),
                &tree,
                &[local_commit, remote_commit],
            )?;

            self.git_repo.cleanup_state()?;

            Ok(MergeResult::Merged)
        } else {
            // There are conflicts
            Ok(MergeResult::Conflicts)
        }
    }

    pub fn get_remote_name_for_branch(&self, branch_name: &str) -> Result<String> {
        if let Some(tracking) = self.tracking_branch(branch_name)? {
            Ok(tracking.remote)
        } else {
            // Fall back to origin if no tracking branch is configured
            Ok("origin".to_string())
        }
    }

    /// Get the ahead/behind count relative to the remote tracking branch
    pub fn get_remote_comparison(
        &self,
        branch_name: &str,
    ) -> Result<Option<RemoteComparison>> {
        // Get the tracking branch info
        let tracking = match self.tracking_branch(branch_name)? {
            Some(tracking) => tracking,
            None => return Ok(None), // No tracking branch configured
        };

        // Check if remote branch exists
        let remote_oid = match self.git_repo.refname_to_id(&tracking.remote_ref) {
            Ok(oid) => oid,
            Err(_) => {
                // Remote branch doesn't exist
                return Ok(Some(RemoteComparison::NoRemote));
            },
        };

        // Get local branch OID
        let local_oid = self.git_repo.head()?.peel_to_commit()?.id();

        // If they're the same, we're up to date
        if local_oid == remote_oid {
            return Ok(Some(RemoteComparison::UpToDate));
        }

        // Calculate ahead/behind using git's graph functions
        let (ahead, behind) =
            self.git_repo.graph_ahead_behind(local_oid, remote_oid)?;

        if ahead > 0 && behind > 0 {
            Ok(Some(RemoteComparison::Diverged(ahead, behind)))
        } else if ahead > 0 {
            Ok(Some(RemoteComparison::Ahead(ahead)))
        } else if behind > 0 {
            Ok(Some(RemoteComparison::Behind(behind)))
        } else {
            Ok(Some(RemoteComparison::UpToDate))
        }
    }

    pub fn remote_callbacks(&self) -> Result<git2::RemoteCallbacks<'static>> {
        self.remote_callbacks_impl(false)
    }

    pub fn remote_callbacks_verbose(&self) -> Result<git2::RemoteCallbacks<'static>> {
        self.remote_callbacks_impl(true)
    }

    fn remote_callbacks_impl(
        &self,
        verbose: bool,
    ) -> Result<git2::RemoteCallbacks<'static>> {
        let config = self.git_repo.config()?;

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(move |url, username_from_url, allowed| {
            if verbose {
                eprintln!("DEBUG: Credential callback invoked");
                eprintln!("  URL: {}", url);
                eprintln!("  Username from URL: {:?}", username_from_url);
                eprintln!("  Allowed types: {:?}", allowed);
            }

            // Try SSH key from agent (only if SSH_AUTH_SOCK is set)
            if allowed.contains(git2::CredentialType::SSH_KEY) {
                if let Some(username) = username_from_url {
                    // Check if SSH agent is actually available
                    if std::env::var("SSH_AUTH_SOCK").is_ok() {
                        if verbose {
                            eprintln!(
                                "  Attempting: SSH key from agent for user '{}'",
                                username
                            );
                        }
                        match git2::Cred::ssh_key_from_agent(username) {
                            Ok(cred) => {
                                if verbose {
                                    eprintln!("  SUCCESS: SSH key from agent");
                                }
                                return Ok(cred);
                            },
                            Err(e) => {
                                if verbose {
                                    eprintln!("  FAILED: SSH key from agent - {}", e);
                                }
                            },
                        }
                    } else if verbose {
                        eprintln!(
                            "  SKIPPED: SSH key from agent (SSH_AUTH_SOCK not set)"
                        );
                    }
                } else if verbose {
                    eprintln!("  SKIPPED: SSH key from agent (no username provided)");
                }

                // Try SSH key files directly
                if let Some(username) = username_from_url
                    && let Ok(home) = std::env::var("HOME")
                {
                    let key_paths = vec![
                        format!("{}/.ssh/id_ed25519", home),
                        format!("{}/.ssh/id_rsa", home),
                        format!("{}/.ssh/id_ecdsa", home),
                    ];

                    for key_path in key_paths {
                        if path::Path::new(&key_path).exists() {
                            if verbose {
                                eprintln!("  Attempting: SSH key file at {}", key_path);
                            }
                            match git2::Cred::ssh_key(
                                username,
                                None, // no public key path
                                path::Path::new(&key_path),
                                None, // no passphrase
                            ) {
                                Ok(cred) => {
                                    if verbose {
                                        eprintln!("  SUCCESS: SSH key file");
                                    }
                                    return Ok(cred);
                                },
                                Err(e) => {
                                    if verbose {
                                        eprintln!("  FAILED: SSH key file - {}", e);
                                    }
                                },
                            }
                        }
                    }
                }
            }

            // Try credential helper
            if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT)
                || allowed.contains(git2::CredentialType::SSH_KEY)
                || allowed.contains(git2::CredentialType::DEFAULT)
            {
                if verbose {
                    eprintln!("  Attempting: Credential helper");
                }
                match git2::Cred::credential_helper(&config, url, username_from_url) {
                    Ok(cred) => {
                        if verbose {
                            eprintln!("  SUCCESS: Credential helper");
                        }
                        return Ok(cred);
                    },
                    Err(e) => {
                        if verbose {
                            eprintln!("  FAILED: Credential helper - {}", e);
                        }
                    },
                }
            }

            // Try username only
            if allowed.contains(git2::CredentialType::USERNAME) {
                let username = username_from_url.unwrap_or("git");
                if verbose {
                    eprintln!("  Attempting: Username only ('{}')", username);
                }
                match git2::Cred::username(username) {
                    Ok(cred) => {
                        if verbose {
                            eprintln!("  SUCCESS: Username");
                        }
                        return Ok(cred);
                    },
                    Err(e) => {
                        if verbose {
                            eprintln!("  FAILED: Username - {}", e);
                        }
                    },
                }
            }

            // Try default
            if verbose {
                eprintln!("  Attempting: Default credentials");
            }
            match git2::Cred::default() {
                Ok(cred) => {
                    if verbose {
                        eprintln!("  SUCCESS: Default credentials");
                    }
                    Ok(cred)
                },
                Err(e) => {
                    if verbose {
                        eprintln!("  FAILED: All credential methods exhausted");
                        eprintln!("  Last error: {}", e);
                    }
                    Err(e)
                },
            }
        });

        Ok(callbacks)
    }

    fn resolve_reference(&self, short_name: &str) -> Result<String> {
        Ok(self
            .git_repo
            .resolve_reference_from_short_name(short_name)?
            .name()
            .with_context(|| {
                format!(
                    "Cannot resolve head reference for repo at `{}`",
                    self.work_dir.display()
                )
            })?
            .to_owned())
    }

    pub fn tracking_branch(&self, branch_name: &str) -> Result<Option<TrackingBranch>> {
        let config = self.git_repo.config()?;

        let remote_key = format!("branch.{}.remote", branch_name);
        let merge_key = format!("branch.{}.merge", branch_name);

        let remote = match config.get_string(&remote_key) {
            Ok(name) => name,
            Err(err) if err.code() == git2::ErrorCode::NotFound => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        let merge_ref = match config.get_string(&merge_key) {
            Ok(name) => name,
            Err(err) if err.code() == git2::ErrorCode::NotFound => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        let branch_short = merge_ref
            .strip_prefix("refs/heads/")
            .unwrap_or(&merge_ref)
            .to_owned();

        let remote_ref = format!("refs/remotes/{}/{}", remote, branch_short);

        Ok(Some(TrackingBranch { remote, remote_ref }))
    }

    fn get_pull_strategy(&self, branch_name: &str) -> Result<PullStrategy> {
        let config = self.git_repo.config()?;

        // First check branch-specific rebase setting (highest priority)
        let branch_rebase_key = format!("branch.{}.rebase", branch_name);
        if let Ok(value) = config.get_string(&branch_rebase_key) {
            return Ok(parse_rebase_config(&value));
        }

        // Then check global pull.rebase setting
        if let Ok(value) = config.get_string("pull.rebase") {
            return Ok(parse_rebase_config(&value));
        }

        // Try as boolean for backward compatibility
        if let Ok(value) = config.get_bool("pull.rebase") {
            return Ok(if value {
                PullStrategy::Rebase
            } else {
                PullStrategy::Merge
            });
        }

        // Default to merge
        Ok(PullStrategy::Merge)
    }

    fn is_worktree_clean(&self) -> Result<bool> {
        let mut status_options = StatusOptions::new();
        status_options.include_ignored(false);
        status_options.include_untracked(true);
        let statuses = self.git_repo.statuses(Some(&mut status_options))?;
        Ok(statuses.is_empty())
    }
}

impl fmt::Debug for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Repo")
            .field("work_dir", &self.work_dir)
            .field("head", &self.head)
            .field("subrepos", &self.subrepos)
            .finish()
    }
}

pub struct TrackingBranch {
    pub remote: String,
    pub remote_ref: String,
}

#[derive(Debug, Clone, PartialEq)]
enum PullStrategy {
    Merge,
    Rebase,
}

fn parse_rebase_config(value: &str) -> PullStrategy {
    match value.to_lowercase().as_str() {
        "true" | "interactive" | "i" | "merges" | "m" => PullStrategy::Rebase,
        "false" => PullStrategy::Merge,
        _ => PullStrategy::Merge, // Default to merge for unknown values
    }
}
