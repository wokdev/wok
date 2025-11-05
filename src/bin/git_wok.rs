use anyhow::{Context, Result, anyhow, bail};
use clap::{ArgAction, Parser};
use git_wok as wok;
use std::{env, io::stdout, path};

fn resolve_path(base: &path::Path, value: &path::Path) -> path::PathBuf {
    if value.is_absolute() {
        path::PathBuf::from(value)
    } else {
        base.join(value)
    }
}

#[derive(Debug, Parser)]
#[clap(
    name = "wok",
    about = "Git Wok -- control several git repositories as a single project.",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS")
)]
struct Args {
    /// Git Wok file path.
    #[clap(
        global = true,
        short('f'),
        long,
        value_parser,
        default_value = wok::DEFAULT_CONFIG_NAME,
    )]
    wokfile_path: path::PathBuf,

    /// Enable verbose debugging output for authentication and network operations
    #[clap(global = true, short('v'), long)]
    verbose: bool,

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Parser)]
enum Command {
    /// Inits the wok file in the workspace "umbrella" repo.
    /// Requires the git repo to be inited already.
    /// Introspects existing submodules and adds them to the workspace config
    /// optionally switching them to the same branch.
    Init {},

    /// Assemble a workspace by initializing subrepos and generating config.
    Assemble {
        /// Path to the workspace directory to assemble.
        directory: path::PathBuf,
    },

    /// Generate shell completion script.
    Completion {
        /// Shell to generate completion script for (bash, fish, zsh).
        #[clap(default_value = "bash")]
        shell: wok::cmd::CompletionShell,
    },

    #[clap(flatten)]
    App(App),
}

#[derive(Debug, Parser)]
enum App {
    /// Add an existing submodule to the wok workspace
    Add {
        /// Path of the submodule relative to the umbrella repo
        submodule_path: path::PathBuf,
    },

    /// Remove a submodule from the wok workspace
    #[clap(alias = "rm")]
    Remove {
        /// Path of the submodule relative to the umbrella repo
        submodule_path: path::PathBuf,
    },

    /// Switch repos to current main repo branch with options
    Switch {
        /// Create the branch in repos if it doesn't exist
        #[clap(long)]
        create: bool,

        /// Act on all configured repos
        #[clap(long)]
        all: bool,

        /// Use specified branch name instead of current main repo branch
        #[clap(long)]
        branch: Option<String>,

        /// Specific repos to switch (if not provided, acts on all matching repos)
        repos: Vec<path::PathBuf>,
    },

    /// Lock submodule state by committing current submodule commits
    Lock,

    /// Update submodules to latest changes from remotes
    Update {
        /// Skip creating a commit with submodule updates
        #[clap(long = "no-commit")]
        no_commit: bool,

        /// Include the umbrella repository (enabled by default)
        #[clap(long, action = ArgAction::SetTrue, conflicts_with = "no_umbrella")]
        umbrella: bool,

        /// Skip the umbrella repository
        #[clap(
              long = "no-umbrella",
              action = ArgAction::SetTrue,
              conflicts_with = "umbrella"
          )]
        no_umbrella: bool,
    },

    /// Show subprojects status (clean/dirty, branch info)
    Status {
        /// Fetch from remotes before comparing
        #[clap(long)]
        fetch: bool,
    },

    /// Push changes from configured repos to remotes
    Push {
        /// Set upstream for new branches
        #[clap(short('u'), long)]
        set_upstream: bool,

        /// Act on all configured repos
        #[clap(long)]
        all: bool,

        /// Use specified branch name instead of current main repo branch
        #[clap(long)]
        branch: Option<String>,

        /// Include the umbrella repository (enabled by default)
        #[clap(long, action = ArgAction::SetTrue, conflicts_with = "no_umbrella")]
        umbrella: bool,

        /// Skip the umbrella repository
        #[clap(
            long = "no-umbrella",
            action = ArgAction::SetTrue,
            conflicts_with = "umbrella"
        )]
        no_umbrella: bool,

        /// Specific repos to push (if not provided, acts on all matching repos)
        repos: Vec<path::PathBuf>,
    },

    /// Add tags to repos, show existing tags, sign and push
    Tag {
        /// Create a new tag
        #[clap(long)]
        create: Option<String>,

        /// Sign the tag with GPG
        #[clap(short('s'), long)]
        sign: bool,

        /// Message for annotated tags
        #[clap(short('m'), long)]
        message: Option<String>,

        /// Push tags to remote
        #[clap(long)]
        push: bool,

        /// Act on all configured repos
        #[clap(long)]
        all: bool,

        /// Include the umbrella repository (enabled by default)
        #[clap(long, action = ArgAction::SetTrue, conflicts_with = "no_umbrella")]
        umbrella: bool,

        /// Skip the umbrella repository
        #[clap(
            long = "no-umbrella",
            action = ArgAction::SetTrue,
            conflicts_with = "umbrella"
        )]
        no_umbrella: bool,

        /// Specific repos to tag (if not provided, acts on all matching repos)
        repos: Vec<path::PathBuf>,
    },

    /// Test git authentication for the current repository
    TestAuth,
}

fn resolve_tag_arguments<'a>(
    create: &'a Option<String>,
    all: bool,
    repos: &'a [path::PathBuf],
    config: &wok::config::Config,
) -> Result<(Option<String>, &'a [path::PathBuf])> {
    if create.is_some() {
        if all && !repos.is_empty() {
            bail!("Cannot specify repositories when using --all");
        }
        return Ok((None, repos));
    }

    if all {
        if let Some((first_arg, rest)) = repos.split_first() {
            let tag = first_arg
                .to_str()
                .ok_or_else(|| {
                    anyhow!("Tag name '{}' is not valid UTF-8", first_arg.display())
                })?
                .to_owned();
            return Ok((Some(tag), rest));
        }

        return Ok((None, repos));
    }

    if let Some((first_arg, rest)) = repos.split_first() {
        let matches_repo = config
            .repos
            .iter()
            .any(|config_repo| config_repo.path == *first_arg);

        if matches_repo {
            Ok((None, repos))
        } else {
            let tag = first_arg
                .to_str()
                .ok_or_else(|| {
                    anyhow!("Tag name '{}' is not valid UTF-8", first_arg.display())
                })?
                .to_owned();
            Ok((Some(tag), rest))
        }
    } else {
        Ok((None, repos))
    }
}

fn resolve_include_umbrella(umbrella_flag: bool, no_umbrella_flag: bool) -> bool {
    if umbrella_flag {
        true
    } else {
        !no_umbrella_flag
    }
}

fn main() -> Result<()> {
    let Args { wokfile_path, verbose: _verbose, cmd } = Args::parse();
    let cwd = env::current_dir().context("Cannot access the current directory")?;
    let mut output = stdout();

    match cmd {
        Command::Init {} => {
            let config_path = resolve_path(&cwd, &wokfile_path);

            if config_path.exists() {
                bail!("Git Wok file already exists at `{}`", config_path.display());
            };

            let repo_dir = config_path.parent().with_context(|| {
                format!("Cannot open work dir for `{}`", config_path.display())
            })?;

            let umbrella = wok::repo::Repo::new(repo_dir, None)?;

            wok::cmd::init(&config_path, &umbrella, &mut output)?
        },
        Command::Assemble { directory } => {
            let workspace_dir = resolve_path(&cwd, &directory);

            let config_path = if wokfile_path.is_absolute() {
                wokfile_path.clone()
            } else {
                workspace_dir.join(&wokfile_path)
            };

            wok::cmd::assemble(&workspace_dir, &config_path, &mut output)?
        },
        Command::Completion { shell } => wok::cmd::completion::<Args>(shell)?,
        Command::App(app_cmd) => {
            let config_path = resolve_path(&cwd, &wokfile_path);

            if !config_path.exists() {
                bail!("Git Wok file not found at `{}`", config_path.display());
            };

            let repo_dir = config_path.parent().with_context(|| {
                format!("Cannot open work dir for `{}`", config_path.display())
            })?;

            let umbrella = wok::repo::Repo::new(repo_dir, None)?;

            let mut wok_config = wok::config::Config::load(&config_path)?;

            if match app_cmd {
                App::Add { submodule_path } => {
                    wok::cmd::repo::add(&mut wok_config, &umbrella, &submodule_path)?
                },
                App::Remove { submodule_path } => {
                    wok::cmd::repo::rm(&mut wok_config, &submodule_path)?
                },
                App::Switch {
                    create,
                    all,
                    branch,
                    repos,
                } => wok::cmd::switch(
                    &mut wok_config,
                    &umbrella,
                    &mut output,
                    create,
                    all,
                    branch.as_deref(),
                    &repos,
                )?,
                App::Lock => {
                    wok::cmd::lock(&mut wok_config, &umbrella, &mut output)?;
                    false // Don't save config for lock command
                },
                App::Update {
                    no_commit,
                    umbrella: umbrella_flag,
                    no_umbrella: no_umbrella_flag,
                } => {
                    let include_umbrella =
                        resolve_include_umbrella(umbrella_flag, no_umbrella_flag);
                    wok::cmd::update(
                        &mut wok_config,
                        &umbrella,
                        &mut output,
                        no_commit,
                        include_umbrella,
                    )?;
                    false // Don't save config for update command
                },
                App::Status { fetch } => {
                    wok::cmd::status(&mut wok_config, &umbrella, &mut output, fetch)?;
                    false // Don't save config for status command
                },
                App::Push {
                    set_upstream,
                    all,
                    branch,
                    umbrella: umbrella_flag,
                    no_umbrella: no_umbrella_flag,
                    repos,
                } => {
                    let include_umbrella =
                        resolve_include_umbrella(umbrella_flag, no_umbrella_flag);

                    wok::cmd::push(
                        &mut wok_config,
                        &umbrella,
                        &mut output,
                        set_upstream,
                        all,
                        branch.as_deref(),
                        include_umbrella,
                        &repos,
                    )?;
                    false // Don't save config for push command
                },
                App::Tag {
                    create,
                    sign,
                    message,
                    push,
                    all,
                    umbrella: umbrella_flag,
                    no_umbrella: no_umbrella_flag,
                    repos,
                } => {
                    let (positional_tag, repo_args) =
                        resolve_tag_arguments(&create, all, &repos, &wok_config)?;
                    let tag_name = create.as_deref().or(positional_tag.as_deref());

                    let include_umbrella =
                        resolve_include_umbrella(umbrella_flag, no_umbrella_flag);

                    wok::cmd::tag(
                        &mut wok_config,
                        &umbrella,
                        &mut output,
                        tag_name,
                        sign,
                        message.as_deref(),
                        push,
                        all,
                        include_umbrella,
                        repo_args,
                    )?;
                    false // Don't save config for tag command
                },
                App::TestAuth => {
                    wok::cmd::test_auth(&umbrella, &mut output)?;
                    false // Don't save config for test-auth command
                },
            } {
                wok_config.save(&config_path)?;
            }
        },
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_repo(path: &str) -> wok::config::Config {
        let mut config = wok::config::Config::new();
        config.add_repo(path::Path::new(path), "main");
        config
    }

    #[test]
    fn derive_tag_from_positional_when_all() {
        let config = config_with_repo("api");
        let repos = vec![path::PathBuf::from("v2.0.0")];

        let (positional_tag, remaining) =
            resolve_tag_arguments(&None, true, &repos, &config).unwrap();

        assert_eq!(positional_tag.as_deref(), Some("v2.0.0"));
        assert!(remaining.is_empty());
    }

    #[test]
    fn resolve_include_umbrella_defaults_to_true() {
        assert!(resolve_include_umbrella(false, false));
    }

    #[test]
    fn resolve_include_umbrella_respects_no_flag() {
        assert!(!resolve_include_umbrella(false, true));
    }

    #[test]
    fn resolve_include_umbrella_respects_umbrella_flag() {
        assert!(resolve_include_umbrella(true, false));
    }

    #[test]
    fn resolve_include_umbrella_prefers_explicit_umbrella_over_exclusion() {
        assert!(resolve_include_umbrella(true, true));
    }

    #[test]
    fn allows_explicit_repos_with_all_when_tag_is_positional() {
        let config = config_with_repo("api");
        let repos = vec![path::PathBuf::from("v2.0.0"), path::PathBuf::from("api")];

        let (positional_tag, remaining) =
            resolve_tag_arguments(&None, true, &repos, &config).unwrap();

        assert_eq!(positional_tag.as_deref(), Some("v2.0.0"));
        assert_eq!(remaining, &repos[1..]);
    }

    #[test]
    fn keeps_repo_arguments_for_listing() {
        let config = config_with_repo("api");
        let repos = vec![path::PathBuf::from("api")];

        let (positional_tag, remaining) =
            resolve_tag_arguments(&None, false, &repos, &config).unwrap();

        assert!(positional_tag.is_none());
        assert_eq!(remaining, repos.as_slice());
    }

    #[test]
    fn rejects_repos_with_all_when_create_present() {
        let config = config_with_repo("api");
        let repos = vec![path::PathBuf::from("api")];
        let create = Some(String::from("v2.0.0"));

        let result = resolve_tag_arguments(&create, true, &repos, &config);
        assert!(result.is_err());
    }

    #[test]
    fn derives_tag_from_first_non_repo_argument() {
        let config = config_with_repo("api");
        let repos = vec![path::PathBuf::from("v2.0.0"), path::PathBuf::from("api")];

        let (positional_tag, remaining) =
            resolve_tag_arguments(&None, false, &repos, &config).unwrap();

        assert_eq!(positional_tag.as_deref(), Some("v2.0.0"));
        assert_eq!(remaining, &repos[1..]);
    }

    #[test]
    fn allows_multiple_repos_with_all_when_no_create() {
        let mut config = config_with_repo("api");
        config.add_repo(path::Path::new("docs"), "main");
        let repos = vec![
            path::PathBuf::from("v2.0.0"),
            path::PathBuf::from("api"),
            path::PathBuf::from("docs"),
        ];

        let (positional_tag, remaining) =
            resolve_tag_arguments(&None, true, &repos, &config).unwrap();

        assert_eq!(positional_tag.as_deref(), Some("v2.0.0"));
        assert_eq!(remaining, &repos[1..]);
    }
}
