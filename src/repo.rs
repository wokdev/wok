use std::{fmt, path};

use anyhow::*;
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

impl Repo {
    pub fn new(work_dir: &path::Path, head_name: Option<&str>) -> Result<Self> {
        let git_repo = git2::Repository::open(work_dir)
            .with_context(|| format!("Cannot open repo at `{}`", work_dir.display()))?;

        let head = match head_name {
            Some(name) => String::from(name),
            None => {
                if git_repo.head_detached().with_context(|| {
                    format!(
                        "Cannot determine head state for repo at `{}`",
                        work_dir.display()
                    )
                })? {
                    bail!(
                        "Cannot operate on a detached head for repo at `{}`",
                        work_dir.display()
                    )
                }

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
            .map(|submodule| Repo::new(&work_dir.join(submodule.path()), Some(&head)))
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

    pub fn switch(&self, head: &str) -> Result<()> {
        self.git_repo.set_head(&self.resolve_reference(head)?)?;
        self.git_repo.checkout_head(None)?;
        Ok(())
    }

    pub fn fetch(&self) -> Result<()> {
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
                            "Failed to fetch from remote '{}' for repo at `{}`",
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
            return Ok(MergeResult::FastForward);
        }

        // Determine pull strategy from git config
        let pull_strategy = self.get_pull_strategy(branch_name)?;

        match pull_strategy {
            PullStrategy::Rebase => {
                // Perform rebase
                self.rebase(branch_name, &remote_commit)
            },
            PullStrategy::Merge => {
                // Perform merge (existing logic)
                self.do_merge(branch_name, &local_commit, &remote_commit, &tracking)
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
        let config = self.git_repo.config()?;

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(move |url, username_from_url, allowed| {
            if allowed.contains(git2::CredentialType::SSH_KEY)
                && let Some(username) = username_from_url
                && let Ok(cred) = git2::Cred::ssh_key_from_agent(username)
            {
                return Ok(cred);
            }

            if (allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT)
                || allowed.contains(git2::CredentialType::SSH_KEY)
                || allowed.contains(git2::CredentialType::DEFAULT))
                && let Ok(cred) =
                    git2::Cred::credential_helper(&config, url, username_from_url)
            {
                return Ok(cred);
            }

            if allowed.contains(git2::CredentialType::USERNAME) {
                if let Some(username) = username_from_url {
                    return git2::Cred::username(username);
                } else {
                    return git2::Cred::username("git");
                }
            }

            git2::Cred::default()
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
