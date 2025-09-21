use std::{fmt, path};

use anyhow::*;
use std::result::Result::Ok;

#[derive(Debug, Clone, PartialEq)]
pub enum MergeResult {
    UpToDate,
    FastForward,
    Merged,
    Conflicts,
}

pub struct Repo {
    pub git_repo: git2::Repository,
    pub work_dir: path::PathBuf,
    pub head: String,
    pub subrepos: Vec<Repo>,
}

impl Repo {
    pub fn new(work_dir: &path::Path, head_name: Option<&str>) -> Result<Self> {
        println!("Reading repo at `{}`", work_dir.display());

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

        println!("Successfully read repo at `{}`", work_dir.display());

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

        let remote_name = self.get_remote_name_for_branch(branch_name)?;

        // Check if remote exists
        match self.git_repo.find_remote(&remote_name) {
            Ok(mut remote) => {
                let refspecs = &[format!(
                    "refs/heads/{}:refs/remotes/{}/{}",
                    branch_name, remote_name, branch_name
                )];

                remote.fetch(refspecs, None, None).with_context(|| {
                    format!(
                        "Failed to fetch from remote '{}' for repo at `{}`",
                        remote_name,
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

    pub fn merge(&self, branch_name: &str) -> Result<MergeResult> {
        // First, fetch the latest changes
        self.fetch()?;

        // Get the remote branch reference
        let remote_name = self.get_remote_name_for_branch(branch_name)?;
        let remote_branch_ref = format!("{}/{}", remote_name, branch_name);

        // Check if remote branch exists
        let remote_branch_oid = match self.git_repo.refname_to_id(&remote_branch_ref) {
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

        // Check if we can fast-forward
        if self
            .git_repo
            .graph_descendant_of(local_commit.id(), remote_commit.id())?
        {
            // Fast-forward merge
            self.git_repo
                .set_head(&format!("refs/heads/{}", branch_name))?;
            self.git_repo.checkout_head(None)?;
            return Ok(MergeResult::FastForward);
        }

        // Perform a merge
        let mut merge_opts = git2::MergeOptions::new();
        merge_opts.fail_on_conflict(false); // Don't fail on conflicts, we'll handle them

        let _merge_result = self.git_repo.merge_commits(
            &local_commit,
            &remote_commit,
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
                &format!("Merge remote-tracking branch '{}'", remote_branch_ref),
                &tree,
                &[&local_commit, &remote_commit],
            )?;

            Ok(MergeResult::Merged)
        } else {
            // There are conflicts
            Ok(MergeResult::Conflicts)
        }
    }

    fn get_remote_name_for_branch(&self, _branch_name: &str) -> Result<String> {
        // For now, simplify by always using 'origin' as the remote
        // TODO: Implement proper upstream detection
        Ok("origin".to_string())
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
