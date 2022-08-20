use std::{fmt, path};

use anyhow::*;

pub struct Repo {
    git_repo: git2::Repository,
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

    pub fn sync(&self) -> Result<()> {
        self.git_repo.set_head(
            self.git_repo
                .resolve_reference_from_short_name(&self.head)?
                .name()
                .with_context(|| {
                    format!(
                        "Cannot resolve head reference for repo at `{}`",
                        self.work_dir.display()
                    )
                })?,
        )?;
        self.git_repo.checkout_head(None)?;
        Ok(())
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
