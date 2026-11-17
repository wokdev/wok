"""Typer command line for Git Wok."""

from __future__ import annotations

import asyncio
import functools
import sys
from collections.abc import Callable
from enum import StrEnum
from pathlib import Path
from typing import Annotated, TypeVar

import click
import typer
from typer.core import TyperGroup

from wok import __version__
from wok.cmd import assemble as assemble_cmd
from wok.cmd import completion as completion_script
from wok.cmd.auth import check_auth
from wok.cmd.init import init
from wok.cmd.lock import lock
from wok.cmd.push import push
from wok.cmd.repo import add, rm
from wok.cmd.status import status
from wok.cmd.switch import switch
from wok.cmd.tag import tag_create, tag_list, tag_push
from wok.cmd.update import update
from wok.config import Config
from wok.errors import WokError, format_error
from wok.git import GitError
from wok.repo import Repo, init_configured_submodules
from wok.runtime import Runtime, current_runtime, runtime_var

app = typer.Typer(
    add_completion=False,
    no_args_is_help=True,
    pretty_exceptions_enable=False,
    context_settings={"help_option_names": ["-h", "--help"]},
    help="Git Wok -- control several git repositories as a single project.",
)


class ShellName(StrEnum):
    bash = "bash"
    fish = "fish"
    zsh = "zsh"


def _version_callback(value: bool) -> None:
    if value:
        typer.echo(f"wok {__version__}")
        raise typer.Exit()


@app.callback()
def main_callback(
    ctx: typer.Context,
    wokfile_path: Annotated[
        Path,
        typer.Option("-f", "--wokfile-path", help="Git Wok file path."),
    ] = Path("wok.toml"),
    verbose: Annotated[
        bool,
        typer.Option("-v", "--verbose", help="Enable verbose git and SSH tracing."),
    ] = False,
    jobs: Annotated[
        int | None,
        typer.Option(
            "-j", "--jobs", help="How many repositories to operate on at once."
        ),
    ] = None,
    version: Annotated[
        bool,
        typer.Option(
            "--version",
            "-V",
            callback=_version_callback,
            is_eager=True,
            help="Show the version and exit.",
        ),
    ] = False,
) -> None:
    """Git Wok -- control several git repositories as a single project."""
    del version
    ctx.ensure_object(dict)
    runtime_var.set(Runtime(jobs=jobs, verbose=verbose, wokfile=wokfile_path))


def _config_path(path: Path | None = None) -> Path:
    chosen = path if path is not None else current_runtime().wokfile
    if chosen.is_absolute():
        return chosen
    return Path.cwd() / chosen


def _require_config(path: Path) -> Config:
    if not path.exists():
        raise WokError(f"Git Wok file not found at `{path}`")
    return Config.load(path)


def _repo_dir(config_path: Path) -> Path:
    parent = config_path.parent
    if not parent.is_dir():
        raise WokError(f"Cannot open work dir for `{config_path}`")
    return parent


F = TypeVar("F", bound=Callable[..., object])


def _guard(fn: F) -> F:
    @functools.wraps(fn)
    def wrapper(*args: object, **kwargs: object) -> object:
        try:
            return fn(*args, **kwargs)
        except (WokError, GitError) as exc:
            typer.echo(format_error(exc), err=True)
            raise typer.Exit(1) from None

    return wrapper  # type: ignore[return-value]


@app.command("init")
@_guard
def init_cmd() -> None:
    """Inits the wok file in the workspace umbrella repo."""
    config_path = _config_path()
    if config_path.exists():
        raise WokError(f"Git Wok file already exists at `{config_path}`")
    umbrella = Repo.open(_repo_dir(config_path))
    init(config_path, umbrella, sys.stdout)


@app.command()
@_guard
def assemble(
    directory: Annotated[
        Path, typer.Argument(help="Path to the workspace directory to assemble.")
    ],
) -> None:
    """Assemble a workspace by initializing subrepos and generating config."""
    workspace = directory if directory.is_absolute() else Path.cwd() / directory
    wokfile = current_runtime().wokfile
    config_path = wokfile if wokfile.is_absolute() else workspace / wokfile
    assemble_cmd(workspace, config_path, sys.stdout)


@app.command()
@_guard
def completion(
    shell: Annotated[ShellName, typer.Argument()] = ShellName.bash,
) -> None:
    """Generate a shell completion script."""
    command = typer.main.get_command(app)
    sys.stdout.write(completion_script(shell.value, command))


@app.command("add")
@_guard
def add_cmd(
    submodule_path: Annotated[
        Path, typer.Argument(help="Submodule path relative to the umbrella.")
    ],
) -> None:
    """Add an existing submodule to the wok workspace."""
    config_path = _config_path()
    wok_config = _require_config(config_path)
    umbrella = Repo.open(_repo_dir(config_path))
    if add(wok_config, umbrella, submodule_path):
        wok_config.save(config_path)


@app.command("remove")
@_guard
def remove_cmd(
    submodule_path: Annotated[
        Path, typer.Argument(help="Submodule path relative to the umbrella.")
    ],
) -> None:
    """Remove a submodule from the wok workspace."""
    _remove(submodule_path)


@app.command("rm")
@_guard
def rm_cmd(
    submodule_path: Annotated[
        Path, typer.Argument(help="Submodule path relative to the umbrella.")
    ],
) -> None:
    """Remove a submodule from the wok workspace."""
    _remove(submodule_path)


def _remove(submodule_path: Path) -> None:
    config_path = _config_path()
    wok_config = _require_config(config_path)
    if rm(wok_config, submodule_path):
        wok_config.save(config_path)


@app.command("switch")
@_guard
def switch_cmd(
    create: Annotated[
        bool,
        typer.Option("-c", "--create", help="Create the branch if it does not exist."),
    ] = False,
    all_repos: Annotated[
        bool, typer.Option("--all", help="Act on all configured repos.")
    ] = False,
    dirty: Annotated[
        bool,
        typer.Option(
            "-d", "-u", "--dirty", help="Only switch repos with local changes."
        ),
    ] = False,
    branch: Annotated[
        str | None, typer.Option("-b", "--branch", help="Branch to target.")
    ] = None,
    repos: Annotated[list[Path] | None, typer.Argument()] = None,
) -> None:
    """Switch repos to match a target umbrella branch and wokfile state."""
    _reject_all_with_repos(all_repos, repos)
    config_path = _config_path()
    _require_config(config_path)
    umbrella = Repo.open(_repo_dir(config_path))
    effective = asyncio.run(
        _prepare_umbrella(umbrella, branch, create, dirty, config_path)
    )
    umbrella = Repo.open(_repo_dir(config_path))
    wok_config = Config.load(config_path)
    changed = switch(
        wok_config,
        umbrella,
        config_path,
        sys.stdout,
        create,
        all_repos,
        dirty,
        effective,
        list(repos or []),
    )
    if changed:
        wok_config.save(config_path)


async def _prepare_umbrella(
    umbrella: Repo,
    branch: str | None,
    create: bool,
    dirty: bool,
    config_path: Path,
) -> str:
    if branch is None:
        if umbrella.head == "<detached>":
            raise WokError(
                "Umbrella repo is in detached HEAD state; "
                "pass -b <BRANCH> to specify a target branch"
            )
        return umbrella.head
    if dirty:
        await umbrella.switch_or_create_preserving(branch, create)
        reloaded = await Repo.open_async(umbrella.work_dir)
        if reloaded.head == branch:
            relative = _wokfile_relative(config_path, reloaded)
            try:
                await reloaded.checkout_path_from_head(relative)
            except (GitError, WokError):
                return branch
        return branch
    await umbrella.ensure_on_branch_existing_or_remote(branch, create)
    reloaded = await Repo.open_async(umbrella.work_dir)
    await reloaded.checkout_path_from_head(_wokfile_relative(config_path, reloaded))
    return branch


def _wokfile_relative(config_path: Path, umbrella: Repo) -> Path:
    try:
        return config_path.resolve().relative_to(umbrella.work_dir.resolve())
    except ValueError as exc:
        raise WokError(
            f"Wokfile must be inside umbrella repo: `{config_path}`"
        ) from exc


@app.command("lock")
@_guard
def lock_cmd() -> None:
    """Lock submodule state by committing current submodule commits."""
    config_path = _config_path()
    wok_config = _require_config(config_path)
    umbrella = Repo.open(_repo_dir(config_path))
    lock(wok_config, umbrella, sys.stdout)


@app.command("update")
@_guard
def update_cmd(
    no_commit: Annotated[
        bool, typer.Option("--no-commit", help="Skip creating a commit.")
    ] = False,
    umbrella: Annotated[bool, typer.Option("--umbrella/--no-umbrella")] = True,
) -> None:
    """Update submodules to the latest changes from remotes."""
    config_path = _config_path()
    wok_config = _require_config(config_path)
    repo_dir = _repo_dir(config_path)
    asyncio.run(
        init_configured_submodules(repo_dir, [repo.path for repo in wok_config.repos])
    )
    umbrella_repo = Repo.open(repo_dir)
    wok_config = Config.load(config_path)
    update(wok_config, umbrella_repo, sys.stdout, no_commit, umbrella)


@app.command("status")
@_guard
def status_cmd(
    fetch: Annotated[
        bool, typer.Option("--fetch", help="Fetch from remotes before comparing.")
    ] = False,
) -> None:
    """Show subproject status."""
    config_path = _config_path()
    wok_config = _require_config(config_path)
    umbrella = Repo.open(_repo_dir(config_path))
    status(wok_config, umbrella, sys.stdout, fetch)


@app.command("push")
@_guard
def push_cmd(
    set_upstream: Annotated[
        bool,
        typer.Option("-u", "--set-upstream", help="Set upstream for new branches."),
    ] = False,
    all_repos: Annotated[
        bool, typer.Option("--all", help="Act on all configured repos.")
    ] = False,
    branch: Annotated[
        str | None, typer.Option("--branch", help="Branch name to push.")
    ] = None,
    umbrella: Annotated[bool, typer.Option("--umbrella/--no-umbrella")] = True,
    repos: Annotated[list[Path] | None, typer.Argument()] = None,
) -> None:
    """Push changes from configured repos to remotes."""
    _reject_all_with_repos(all_repos, repos)
    config_path = _config_path()
    wok_config = _require_config(config_path)
    umbrella_repo = Repo.open(_repo_dir(config_path))
    push(
        wok_config,
        umbrella_repo,
        sys.stdout,
        set_upstream,
        all_repos,
        branch,
        umbrella,
        list(repos or []),
    )


class TagGroup(TyperGroup):
    """Fall back to implicit tag create when the first token is not a subcommand."""

    def resolve_command(self, ctx: click.Context, args: list[str]):  # pyrefly: ignore
        positional = [arg for arg in args if not arg.startswith("-")]
        if positional and positional[0] not in self.commands:
            if _all_known_repos(ctx, positional):
                args = ["list", *args]
            else:
                args = ["create", *args]
        return super().resolve_command(ctx, args)  # pyrefly: ignore


def _all_known_repos(ctx: click.Context, tokens: list[str]) -> bool:
    parent = ctx.parent
    params = parent.params if parent is not None else {}
    raw = params.get("wokfile_path", Path("wok.toml"))
    path = raw if isinstance(raw, Path) else Path(str(raw))
    if not path.is_absolute():
        path = Path.cwd() / path
    if not path.exists():
        return False
    try:
        config = Config.load(path)
    except WokError:
        return False
    known = {repo.path for repo in config.repos}
    return all(Path(token) in known for token in tokens)


tag_app = typer.Typer(
    cls=TagGroup,
    invoke_without_command=True,
    help="Add tags to repos, show existing tags, sign and push.",
)


def _reject_all_with_repos(all_repos: bool, repos: list[Path] | None) -> None:
    if all_repos and repos:
        raise WokError("Cannot specify repositories when using --all")


@tag_app.callback(invoke_without_command=True)
@_guard
def tag_root(
    ctx: typer.Context,
    all_repos: Annotated[
        bool, typer.Option("--all", help="Act on all configured repos.")
    ] = False,
    umbrella: Annotated[bool, typer.Option("--umbrella/--no-umbrella")] = True,
) -> None:
    """Add tags to repos, show existing tags, sign and push."""
    if ctx.invoked_subcommand is not None:
        return
    _run_tag_list(all_repos, umbrella, [])


@tag_app.command("create")
@_guard
def tag_create_cmd(
    ctx: typer.Context,
    tag_name: Annotated[str, typer.Argument(help="Tag name to create.")],
    sign: Annotated[
        bool, typer.Option("-s", "--sign", help="Sign the tag with GPG.")
    ] = False,
    message: Annotated[
        str | None, typer.Option("-m", "--message", help="Message for annotated tags.")
    ] = None,
    updated: Annotated[
        bool,
        typer.Option("-u", "--updated", help="Only tag repos whose HEAD has no tags."),
    ] = False,
    all_repos: Annotated[
        bool, typer.Option("--all", help="Act on all configured repos.")
    ] = False,
    umbrella: Annotated[bool, typer.Option("--umbrella/--no-umbrella")] = True,
    repos: Annotated[list[Path] | None, typer.Argument()] = None,
) -> None:
    """Create a new tag."""
    parent = _tag_flags(ctx)
    chosen_sign = sign or bool(parent.get("sign"))
    raw_message = parent.get("message")
    if message is not None:
        chosen_message = message
    elif isinstance(raw_message, str):
        chosen_message = raw_message
    else:
        chosen_message = None
    chosen_all = all_repos or bool(parent.get("all_repos"))
    chosen_umbrella = (
        umbrella if "umbrella" not in parent else bool(parent.get("umbrella", umbrella))
    )
    _reject_all_with_repos(chosen_all, repos)
    _run_tag_create(
        tag_name,
        chosen_sign,
        chosen_message,
        chosen_all,
        chosen_umbrella,
        updated,
        list(repos or []),
    )


@tag_app.command("push")
@_guard
def tag_push_cmd(
    ctx: typer.Context,
    all_repos: Annotated[bool, typer.Option("--all")] = False,
    umbrella: Annotated[bool, typer.Option("--umbrella/--no-umbrella")] = True,
    repos: Annotated[list[Path] | None, typer.Argument()] = None,
) -> None:
    """Push tags to remotes."""
    parent = _tag_flags(ctx)
    chosen_all = all_repos or bool(parent.get("all_repos"))
    chosen_umbrella = (
        umbrella if "umbrella" not in parent else bool(parent.get("umbrella", umbrella))
    )
    _reject_all_with_repos(chosen_all, repos)
    _run_tag_push(chosen_all, chosen_umbrella, list(repos or []))


@tag_app.command("list")
@_guard
def tag_list_cmd(
    ctx: typer.Context,
    all_repos: Annotated[bool, typer.Option("--all")] = False,
    umbrella: Annotated[bool, typer.Option("--umbrella/--no-umbrella")] = True,
    repos: Annotated[list[Path] | None, typer.Argument()] = None,
) -> None:
    """List existing tags."""
    parent = _tag_flags(ctx)
    chosen_all = all_repos or bool(parent.get("all_repos"))
    chosen_umbrella = (
        umbrella if "umbrella" not in parent else bool(parent.get("umbrella", umbrella))
    )
    _reject_all_with_repos(chosen_all, repos)
    _run_tag_list(chosen_all, chosen_umbrella, list(repos or []))


def _tag_flags(ctx: typer.Context) -> dict[str, object]:
    parent = ctx.parent
    if parent is None:
        return {}
    return dict(parent.params)


def _run_tag_list(all_repos: bool, umbrella: bool, repos: list[Path]) -> None:
    config_path = _config_path()
    wok_config = _require_config(config_path)
    umbrella_repo = Repo.open(_repo_dir(config_path))
    tag_list(wok_config, umbrella_repo, sys.stdout, all_repos, umbrella, repos)


def _run_tag_create(
    tag_name: str,
    sign: bool,
    message: str | None,
    all_repos: bool,
    umbrella: bool,
    updated: bool,
    repos: list[Path],
) -> None:
    config_path = _config_path()
    wok_config = _require_config(config_path)
    umbrella_repo = Repo.open(_repo_dir(config_path))
    tag_create(
        wok_config,
        umbrella_repo,
        sys.stdout,
        tag_name,
        sign,
        message,
        all_repos,
        umbrella,
        updated,
        repos,
    )


def _run_tag_push(all_repos: bool, umbrella: bool, repos: list[Path]) -> None:
    config_path = _config_path()
    wok_config = _require_config(config_path)
    umbrella_repo = Repo.open(_repo_dir(config_path))
    tag_push(wok_config, umbrella_repo, sys.stdout, all_repos, umbrella, repos)


app.add_typer(tag_app, name="tag")


@app.command("test-auth")
@_guard
def test_auth_cmd() -> None:
    """Test git authentication for the current repository."""
    config_path = _config_path()
    repo_dir = _repo_dir(config_path) if config_path.exists() else Path.cwd()
    check_auth(Repo.open(repo_dir), sys.stdout)


def main() -> None:
    """Console-script entry point."""
    command = typer.main.get_command(app)
    try:
        command(standalone_mode=False)
    except (WokError, GitError) as exc:
        typer.echo(format_error(exc), err=True)
        raise SystemExit(1) from None
    except click.ClickException as exc:
        exc.show()
        raise SystemExit(exc.exit_code) from None
    except typer.Exit as exc:
        raise SystemExit(exc.exit_code) from None
