use std::{fmt, path};

use anyhow::*;

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
