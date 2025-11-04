use anyhow::*;
use std::collections::HashMap;
use std::io::Write;
use std::panic::{self, AssertUnwindSafe};
use std::result::Result::Ok;

use crate::{config, repo};

#[allow(clippy::too_many_arguments)]
pub fn tag<W: Write>(
    wok_config: &mut config::Config,
    umbrella: &repo::Repo,
    stdout: &mut W,
    tag_name: Option<&str>,
    sign: bool,
    message: Option<&str>,
    push: bool,
    all: bool,
    include_umbrella: bool,
    target_repos: &[std::path::PathBuf],
) -> Result<()> {
    // Determine which repos to tag
    let repos_to_tag: Vec<config::Repo> = if all {
        // Tag all configured repos, skipping those opted out unless explicitly targeted
        wok_config
            .repos
            .iter()
            .filter(|config_repo| {
                !config_repo.is_skipped_for("tag")
                    || target_repos.contains(&config_repo.path)
            })
            .cloned()
            .collect()
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
            .filter(|config_repo| {
                config_repo.head == umbrella.head && !config_repo.is_skipped_for("tag")
            })
            .cloned()
            .collect()
    };

    let total_targets = repos_to_tag.len() + usize::from(include_umbrella);

    if total_targets == 0 {
        writeln!(stdout, "No repositories to tag")?;
        return Ok(());
    }

    match tag_name {
        Some(name) => {
            // Create new tag
            writeln!(
                stdout,
                "Creating tag '{}' in {} repositories...",
                name, total_targets
            )?;

            if include_umbrella {
                match create_tag(umbrella, name, sign, message) {
                    Ok(result) => match result {
                        TagResult::Created => {
                            writeln!(stdout, "- 'umbrella': created tag '{}'", name)?;
                        },
                        TagResult::AlreadyExists => {
                            writeln!(
                                stdout,
                                "- 'umbrella': tag '{}' already exists",
                                name
                            )?;
                        },
                    },
                    Err(e) => {
                        writeln!(
                            stdout,
                            "- 'umbrella': failed to create tag '{}' - {}",
                            name, e
                        )?;
                    },
                }
            }

            for config_repo in &repos_to_tag {
                if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
                    match create_tag(subrepo, name, sign, message) {
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
            writeln!(stdout, "Listing tags in {} repositories...", total_targets)?;

            if include_umbrella {
                match list_tags(umbrella) {
                    Ok(tags) => {
                        if tags.is_empty() {
                            writeln!(stdout, "- 'umbrella': no tags found")?;
                        } else {
                            writeln!(stdout, "- 'umbrella': {}", tags.join(", "))?;
                        }
                    },
                    Err(e) => {
                        writeln!(stdout, "- 'umbrella': failed to list tags - {}", e)?;
                    },
                }
            }

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

        if include_umbrella {
            match push_tags(umbrella) {
                Ok(PushResult::Pushed) => {
                    writeln!(stdout, "- 'umbrella': pushed tags")?;
                },
                Ok(PushResult::Skipped) => {
                    writeln!(stdout, "- 'umbrella': no tags to push")?;
                },
                Err(e) => {
                    writeln!(stdout, "- 'umbrella': failed to push tags - {}", e)?;
                },
            }
        }

        for config_repo in &repos_to_tag {
            if let Some(subrepo) = umbrella.get_subrepo_by_path(&config_repo.path) {
                match push_tags(subrepo) {
                    Ok(PushResult::Pushed) => {
                        writeln!(
                            stdout,
                            "- '{}': pushed tags",
                            config_repo.path.display()
                        )?;
                    },
                    Ok(PushResult::Skipped) => {
                        writeln!(
                            stdout,
                            "- '{}': no tags to push",
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
        total_targets
    )?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum TagResult {
    Created,
    AlreadyExists,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PushResult {
    Pushed,
    Skipped,
}

fn create_tag(
    repo: &repo::Repo,
    tag_name: &str,
    sign: bool,
    message: Option<&str>,
) -> Result<TagResult> {
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
    if sign || message.is_some() {
        // Create annotated tag (signed or with message)
        let signature = repo.git_repo.signature()?;
        let default_message = format!("Tag {}", tag_name);
        let tag_message = message.unwrap_or(&default_message);
        let _tag_ref = repo.git_repo.tag(
            tag_name,
            commit_obj,
            &signature,
            tag_message,
            sign, // Pass true for GPG signing, false otherwise
        )?;
    } else {
        // Create lightweight tag (no message, no signature)
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

fn push_tags(repo: &repo::Repo) -> Result<PushResult> {
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

    // Collect explicit tag refspecs; libgit2 does not expand wildcards automatically.
    let tag_names = repo.git_repo.tag_names(None)?;
    if tag_names.is_empty() {
        return Ok(PushResult::Skipped);
    }

    // Discover which tags already exist on the remote so we avoid redundant pushes.
    let connection = remote.connect_auth(
        git2::Direction::Push,
        Some(repo.remote_callbacks()?),
        None,
    )?;

    let remote_tags =
        match panic::catch_unwind(AssertUnwindSafe(|| -> Result<_, git2::Error> {
            let mut tags = HashMap::new();
            for head in connection.list()?.iter() {
                let name = head.name();
                if name.starts_with("refs/tags/") {
                    tags.insert(name.to_string(), head.oid());
                }
            }
            Ok(tags)
        })) {
            Ok(Ok(tags)) => tags,
            Ok(Err(err)) => return Err(err.into()),
            Err(_) => HashMap::new(),
        };
    drop(connection);

    let mut refspecs: Vec<String> = Vec::new();
    for tag_name in tag_names.iter().flatten() {
        let refname = format!("refs/tags/{tag_name}");
        let reference = repo.git_repo.find_reference(&refname)?;
        let target_oid = reference.target().with_context(|| {
            format!("Tag '{}' does not point to an object", tag_name)
        })?;

        match remote_tags.get(&refname) {
            Some(remote_oid) if *remote_oid == target_oid => {
                // Remote already has this tag pointing at the same object.
            },
            _ => refspecs.push(format!("{refname}:{refname}")),
        }
    }

    if refspecs.is_empty() {
        return Ok(PushResult::Skipped);
    }

    let refspec_refs: Vec<&str> =
        refspecs.iter().map(|refspec| refspec.as_str()).collect();
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(repo.remote_callbacks()?);

    let push_result = remote.push(&refspec_refs, Some(&mut push_options));
    let disconnect_result = remote.disconnect();
    push_result?;
    disconnect_result?;

    Ok(PushResult::Pushed)
}
