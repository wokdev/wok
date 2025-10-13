use crate::repo;
use anyhow::*;
use git2::{ErrorCode, Repository, RepositoryInitOptions, Signature};
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::{self, PathBuf};
use std::process::Command;

const INITIAL_COMMIT_MESSAGE: &str = "Initial commit (wok assemble)";
const DEFAULT_AUTHOR_NAME: &str = "wok assemble";
const DEFAULT_AUTHOR_EMAIL: &str = "assemble@wok.dev";

pub fn assemble<W: Write>(
    workspace_dir: &path::Path,
    config_path: &path::Path,
    stdout: &mut W,
) -> Result<()> {
    if !workspace_dir.exists() {
        bail!(
            "Workspace directory `{}` does not exist",
            workspace_dir.display()
        );
    }
    if !workspace_dir.is_dir() {
        bail!(
            "Workspace path `{}` is not a directory",
            workspace_dir.display()
        );
    }

    writeln!(
        stdout,
        "Assembling workspace in `{}`",
        workspace_dir.display()
    )?;

    let mut workspace_repo =
        ensure_git_repo(workspace_dir, true).with_context(|| {
            format!("Cannot prepare repo at `{}`", workspace_dir.display())
        })?;
    let mut submodule_paths = current_submodule_paths(&workspace_repo)?;

    for entry in std::fs::read_dir(workspace_dir).with_context(|| {
        format!(
            "Cannot read workspace directory at `{}`",
            workspace_dir.display()
        )
    })? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_dir() {
            continue;
        }

        let name = entry.file_name();
        if name == OsStr::new(".git") || name == OsStr::new(".gitmodules") {
            continue;
        }

        let entry_path = entry.path();
        let rel_path = entry_path.strip_prefix(workspace_dir).with_context(|| {
            format!("Cannot derive relative path for `{}`", entry_path.display())
        })?;

        let rel_path = rel_path.to_path_buf();

        let child_repo = ensure_git_repo(&entry_path, true).with_context(|| {
            format!(
                "Cannot prepare component repo at `{}`",
                entry_path.display()
            )
        })?;

        ensure_initial_commit(&child_repo)?;

        if !submodule_paths.contains(&rel_path) {
            let source_path = entry_path.canonicalize().with_context(|| {
                format!("Cannot resolve path `{}`", entry_path.display())
            })?;

            register_submodule(workspace_dir, &rel_path, &source_path)?;

            if let Some(remote_url) = repo_remote_url(&child_repo)? {
                update_submodule_remote(workspace_dir, &rel_path, &remote_url)?;
            }

            writeln!(stdout, "Registered `{}` as submodule", rel_path.display())?;

            workspace_repo = Repository::open(workspace_dir)?;
            submodule_paths = current_submodule_paths(&workspace_repo)?;
        }
    }

    let umbrella = repo::Repo::new(workspace_dir, None)?;
    super::init::init(config_path, &umbrella, stdout)?;

    Ok(())
}

fn ensure_git_repo(path: &path::Path, ensure_commit: bool) -> Result<Repository> {
    let repo = match Repository::open(path) {
        std::result::Result::Ok(repo) => repo,
        Err(_) => {
            let mut opts = RepositoryInitOptions::new();
            opts.initial_head("main");
            Repository::init_opts(path, &opts).with_context(|| {
                format!("Cannot init git repo at `{}`", path.display())
            })?
        },
    };

    if ensure_commit {
        ensure_initial_commit(&repo)?;
    }

    Ok(repo)
}

fn ensure_initial_commit(repo: &Repository) -> Result<()> {
    match repo.head() {
        std::result::Result::Ok(_) => Ok(()),
        Err(err)
            if err.code() == ErrorCode::UnbornBranch
                || err.code() == ErrorCode::NotFound =>
        {
            let signature = Signature::now(DEFAULT_AUTHOR_NAME, DEFAULT_AUTHOR_EMAIL)
                .context("Cannot create signature for initial commit")?;

            let tree_id = {
                let mut index = repo.index()?;
                index.write_tree()?
            };

            let tree = repo.find_tree(tree_id)?;

            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                INITIAL_COMMIT_MESSAGE,
                &tree,
                &[],
            )
            .context("Cannot create initial commit")?;

            Ok(())
        },
        Err(err) => Err(err.into()),
    }
}

fn current_submodule_paths(repo: &Repository) -> Result<HashSet<PathBuf>> {
    Ok(repo
        .submodules()
        .with_context(|| {
            format!(
                "Cannot list submodules for repo at `{}`",
                repo.workdir()
                    .unwrap_or_else(|| path::Path::new("<unknown>"))
                    .display()
            )
        })?
        .into_iter()
        .map(|submodule| submodule.path().to_path_buf())
        .collect())
}

fn register_submodule(
    workspace_dir: &path::Path,
    rel_path: &path::Path,
    source_path: &path::Path,
) -> Result<()> {
    run_git(
        workspace_dir,
        [
            OsStr::new("submodule"),
            OsStr::new("add"),
            source_path.as_os_str(),
            rel_path.as_os_str(),
        ],
    )
    .with_context(|| format!("Cannot add `{}` as submodule", rel_path.display()))?;

    run_git(
        workspace_dir,
        [
            OsStr::new("submodule"),
            OsStr::new("absorbgitdirs"),
            rel_path.as_os_str(),
        ],
    )
    .with_context(|| {
        format!(
            "Cannot absorb git dir for submodule `{}`",
            rel_path.display()
        )
    })?;

    Ok(())
}

fn update_submodule_remote(
    workspace_dir: &path::Path,
    rel_path: &path::Path,
    remote_url: &str,
) -> Result<()> {
    let key_os =
        OsString::from(format!("submodule.{}.url", rel_path.to_string_lossy()));
    let remote_os = OsString::from(remote_url);

    run_git(
        workspace_dir,
        [
            OsStr::new("config"),
            OsStr::new("-f"),
            OsStr::new(".gitmodules"),
            key_os.as_os_str(),
            remote_os.as_os_str(),
        ],
    )?;

    run_git(
        workspace_dir,
        [
            OsStr::new("config"),
            key_os.as_os_str(),
            remote_os.as_os_str(),
        ],
    )?;

    Ok(())
}

fn repo_remote_url(repo: &Repository) -> Result<Option<String>> {
    match repo.find_remote("origin") {
        std::result::Result::Ok(remote) => Ok(remote.url().map(|url| url.to_string())),
        Err(err) if err.code() == ErrorCode::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

fn run_git<I, S>(cwd: &path::Path, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .status()
        .with_context(|| format!("Cannot execute git in `{}`", cwd.display()))?;

    if !status.success() {
        bail!("Git command failed in `{}`", cwd.display());
    }

    Ok(())
}
