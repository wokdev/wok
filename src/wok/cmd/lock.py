"""Record submodule pointers with git commit."""

from __future__ import annotations

import asyncio
from typing import TextIO

from wok.config import Config
from wok.errors import WokError
from wok.git import GitError, git
from wok.repo import Repo, submodule_paths, truncate_subject


def lock(wok_config: Config, umbrella: Repo, stdout: TextIO) -> None:
    asyncio.run(_lock(wok_config, umbrella, stdout))


async def _lock(wok_config: Config, umbrella: Repo, stdout: TextIO) -> None:
    for config_repo in wok_config.repos:
        subrepo = umbrella.get_subrepo_by_path(config_repo.path)
        if subrepo is not None:
            await subrepo.switch(config_repo.head)

    changed = await _stage_submodules(umbrella)
    if not changed:
        stdout.write("No submodule changes detected; nothing to lock\n")
        return

    message = await _lock_message(umbrella)
    try:
        await git(["commit", "-F", "-"], umbrella.work_dir, input_text=message)
    except GitError as exc:
        raise WokError(f"Cannot lock workspace in `{umbrella.work_dir}`") from exc
    stdout.write("Locked submodule state\n")


async def _stage_submodules(umbrella: Repo) -> bool:
    paths = [path.as_posix() for path in await submodule_paths(umbrella.work_dir)]
    if paths:
        await git(["add", "--", *paths], umbrella.work_dir)
    output = await git(["diff", "--cached", "--name-only"], umbrella.work_dir)
    return bool(output.strip())


async def _lock_message(umbrella: Repo) -> str:
    output = await git(["diff", "--cached", "--name-only", "-z"], umbrella.work_dir)
    names = [name for name in output.split("\0") if name]
    changed: list[tuple[str, str]] = []
    known = {path.as_posix() for path in await submodule_paths(umbrella.work_dir)}
    for name in names:
        if name not in known:
            continue
        subrepo = umbrella.get_subrepo_by_path(umbrella.work_dir / name)
        if subrepo is None:
            continue
        subject = truncate_subject(await subrepo.commit_message_subject())
        changed.append((name, subject))
    message = "Lock submodule state"
    if changed:
        message += "\n\nChanged submodules:"
        for name, subject in changed:
            message += f"\n- {name}: {subject}"
    return message + "\n"


async def stage_and_commit(umbrella: Repo, message: str) -> bool:
    """Stage initialized submodules and commit when the index changed."""
    changed = await _stage_submodules(umbrella)
    if not changed:
        return False
    try:
        await git(["commit", "-F", "-"], umbrella.work_dir, input_text=message)
    except GitError as exc:
        raise WokError(f"Cannot commit workspace in `{umbrella.work_dir}`") from exc
    return True
