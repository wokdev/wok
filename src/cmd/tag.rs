use anyhow::*;
use std::io::Write;
use std::result::Result::Ok;

use crate::{config, repo};

#[allow(clippy::too_many_arguments)]
pub fn tag<W: Write>(
    wok_config: &mut config::Config,
    umbrella: &repo::Repo,
    stdout: &mut W,
    tag_name: Option<&str>,
    sign: bool,
    push: bool,
    all: bool,
    target_repos: &[std::path::PathBuf],
) -> Result<()> {
    // Determine which repos to tag
    let repos_to_tag = if all {
        // Tag all configured repos
        wok_config.repos.clone()
    } else if !target_repos.is_empty() {
        // Tag only specified repos
        wok_config
            .repos
            .iter()
            .filter(|config_repo| target_repos.contains(&config_repo.path))
            .cloned()
            .collect()
    } else {
        // Tag repos that match the current main repo branch
        wok_config
            .repos
            .iter()
            .filter(|config_repo| config_repo.head == umbrella.head)
            .cloned()
            .collect()
    };

    if repos_to_tag.is_empty() {
        writeln!(stdout, "No repositories to tag")?;
        return Ok(());
    }

    match tag_name {
        Some(name) => {
            // Create new tag
            writeln!(
                stdout,
                "Creating tag '{}' in {} repositories...",
                name,
                repos_to_tag.len()
            )?;

            for config_repo in &repos_to_tag {
                if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
                    match create_tag(subrepo, name, sign) {
                        Ok(result) => match result {
                            TagResult::Created => {
                                writeln!(
                                    stdout,
                                    "- '{}': created tag '{}'",
                                    config_repo.path.display(),
                                    name
                                )?;
                            },
                            TagResult::AlreadyExists => {
                                writeln!(
                                    stdout,
                                    "- '{}': tag '{}' already exists",
                                    config_repo.path.display(),
                                    name
                                )?;
                            },
                        },
                        Err(e) => {
                            writeln!(
                                stdout,
                                "- '{}': failed to create tag '{}' - {}",
                                config_repo.path.display(),
                                name,
                                e
                            )?;
                        },
                    }
                }
            }
        },
        None => {
            // List existing tags
            writeln!(
                stdout,
                "Listing tags in {} repositories...",
                repos_to_tag.len()
            )?;

            for config_repo in &repos_to_tag {
                if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
                    match list_tags(subrepo) {
                        Ok(tags) => {
                            if tags.is_empty() {
                                writeln!(
                                    stdout,
                                    "- '{}': no tags found",
                                    config_repo.path.display()
                                )?;
                            } else {
                                writeln!(
                                    stdout,
                                    "- '{}': {}",
                                    config_repo.path.display(),
                                    tags.join(", ")
                                )?;
                            }
                        },
                        Err(e) => {
                            writeln!(
                                stdout,
                                "- '{}': failed to list tags - {}",
                                config_repo.path.display(),
                                e
                            )?;
                        },
                    }
                }
            }
        },
    }

    // Push tags if requested
    if push {
        writeln!(stdout, "Pushing tags to remotes...")?;
        for config_repo in &repos_to_tag {
            if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
                match push_tags(subrepo) {
                    Ok(_) => {
                        writeln!(
                            stdout,
                            "- '{}': pushed tags",
                            config_repo.path.display()
                        )?;
                    },
                    Err(e) => {
                        writeln!(
                            stdout,
                            "- '{}': failed to push tags - {}",
                            config_repo.path.display(),
                            e
                        )?;
                    },
                }
            }
        }
    }

    writeln!(
        stdout,
        "Successfully processed {} repositories",
        repos_to_tag.len()
    )?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum TagResult {
    Created,
    AlreadyExists,
}

fn create_tag(repo: &repo::Repo, tag_name: &str, sign: bool) -> Result<TagResult> {
    // Check if tag already exists by trying to find it
    if repo
        .git_repo
        .revparse_single(&format!("refs/tags/{}", tag_name))
        .is_ok()
    {
        return Ok(TagResult::AlreadyExists);
    }

    // Get the current HEAD commit
    let head = repo.git_repo.head()?;
    let commit = head.peel_to_commit()?;
    let commit_obj = commit.as_object();

    // Create the tag
    if sign {
        // Create signed tag
        let signature = repo.git_repo.signature()?;
        let _tag_ref = repo.git_repo.tag(
            tag_name,
            commit_obj,
            &signature,
            &format!("Tag {}", tag_name),
            false,
        )?;
    } else {
        // Create lightweight tag
        let _tag_ref = repo.git_repo.tag_lightweight(tag_name, commit_obj, false)?;
    }

    Ok(TagResult::Created)
}

fn list_tags(repo: &repo::Repo) -> Result<Vec<String>> {
    let mut tags = Vec::new();

    // Get all tag references
    let tag_names = repo.git_repo.tag_names(None)?;

    for tag_name in tag_names.iter().flatten() {
        tags.push(tag_name.to_string());
    }

    // Sort tags for consistent output
    tags.sort();

    Ok(tags)
}

fn push_tags(repo: &repo::Repo) -> Result<()> {
    // Get the remote name for the current branch
    let head_ref = repo.git_repo.head()?;
    let branch_name = head_ref.shorthand().with_context(|| {
        format!(
            "Cannot get branch name for repo at `{}`",
            repo.work_dir.display()
        )
    })?;

    let remote_name = repo.get_remote_name_for_branch(branch_name)?;

    // Check if remote exists
    let mut remote = match repo.git_repo.find_remote(&remote_name) {
        Ok(remote) => remote,
        Err(_) => {
            return Err(anyhow!("No remote '{}' configured", remote_name));
        },
    };

    // Push all tags
    let refspecs = &["refs/tags/*:refs/tags/*"];
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(repo.remote_callbacks()?);

    remote.push(refspecs, Some(&mut push_options))?;

    Ok(())
}
