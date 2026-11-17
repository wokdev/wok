"""Pull configured repositories, then commit submodule pointers."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from typing import TextIO

from wok.config import Config
from wok.config import Repo as ConfigRepo
from wok.errors import WokError
from wok.git import GitError, git
from wok.repo import MergeResult, Repo
from wok.runtime import resolve_jobs
from wok.ui import run_parallel


@dataclass
class UpdateLine:
    text: str
    updated: bool
    conflicts: bool
    name: str
    branch: str
    commit: str


def update(
    wok_config: Config,
    umbrella: Repo,
    stdout: TextIO,
    no_commit: bool,
    include_umbrella: bool,
) -> None:
    asyncio.run(_update(wok_config, umbrella, stdout, no_commit, include_umbrella))


async def _update(
    wok_config: Config,
    umbrella: Repo,
    stdout: TextIO,
    no_commit: bool,
    include_umbrella: bool,
) -> None:
    stdout.write("Updating repositories...\n")
    selected = [repo for repo in wok_config.repos if not repo.is_skipped_for("update")]

    async def update_sub(config_repo: ConfigRepo) -> UpdateLine:
        subrepo = umbrella.get_subrepo_by_path(config_repo.path)
        label = config_repo.path.as_posix()
        branch = umbrella.head if not config_repo.head.strip() else config_repo.head
        if subrepo is None:
            return UpdateLine(
                f"- '{label}': skipped '{branch}': missing submodule",
                False,
                False,
                label,
                branch,
                "",
            )
        return await _update_repo(subrepo, branch, label, allow_dirty=False)

    results = await run_parallel(
        selected,
        update_sub,
        resolve_jobs(len(selected) or 1),
        label=lambda config_repo: config_repo.path.as_posix(),
    )
    saw_updates = False
    saw_conflicts = False
    updated: list[tuple[str, str, str]] = []
    for result in results:
        if isinstance(result, BaseException):
            raise result
        stdout.write(result.text + "\n")
        saw_updates = saw_updates or result.updated
        saw_conflicts = saw_conflicts or result.conflicts
        if result.updated:
            updated.append((result.name, result.branch, result.commit))

    if include_umbrella:
        umbrella_line = await _update_repo(
            umbrella, umbrella.head, "umbrella", allow_dirty=True
        )
        stdout.write(umbrella_line.text + "\n")
        saw_conflicts = saw_conflicts or umbrella_line.conflicts

    if saw_conflicts:
        stdout.write("Skipped committing umbrella repo due to merge conflicts\n")
        return

    staged = await _stage_submodules(umbrella)
    if no_commit:
        if staged or saw_updates:
            stdout.write(
                "Changes staged; commit skipped because --no-commit was provided\n"
            )
        else:
            stdout.write("No submodule updates detected; nothing to commit\n")
        return
    if not staged:
        stdout.write("No submodule updates detected; nothing to commit\n")
        return
    message = await _update_message(umbrella, updated)
    await git(["commit", "-F", "-"], umbrella.work_dir, input_text=message)
    stdout.write("Updated submodule state committed\n")


async def _update_repo(
    repo: Repo, branch: str, label: str, *, allow_dirty: bool
) -> UpdateLine:
    try:
        if allow_dirty:
            if repo.head != branch:
                await repo.switch(branch)
        else:
            await repo.ensure_on_branch(branch)
    except (WokError, GitError) as exc:
        return UpdateLine(
            f"- '{label}': skipped '{branch}': {exc}", False, False, label, branch, ""
        )

    result = await repo.pull(branch)
    commit = await repo.head_commit()
    short = commit[:8]
    if result is MergeResult.UP_TO_DATE:
        text = f"- '{label}': already up to date on '{branch}' ({short})"
        updated = False
    elif result is MergeResult.FAST_FORWARD:
        text = f"- '{label}': fast-forwarded '{branch}' to {short}"
        updated = True
    elif result is MergeResult.MERGED:
        text = f"- '{label}': merged '{branch}' to {short}"
        updated = True
    elif result is MergeResult.REBASED:
        text = f"- '{label}': rebased '{branch}' to {short}"
        updated = True
    else:
        text = (
            f"- '{label}': merge conflicts in '{branch}' ({short}), "
            "manual resolution required"
        )
        return UpdateLine(text, False, True, label, branch, commit)
    return UpdateLine(text, updated, False, label, branch, commit)


async def _stage_submodules(umbrella: Repo) -> bool:
    from wok.repo import submodule_paths

    paths = [path.as_posix() for path in await submodule_paths(umbrella.work_dir)]
    if not paths:
        return False
    await git(["add", "--", *paths], umbrella.work_dir)
    output = await git(["diff", "--cached", "--name-only"], umbrella.work_dir)
    return bool(output.strip())


async def _update_message(umbrella: Repo, updated: list[tuple[str, str, str]]) -> str:
    output = await git(["diff", "--cached", "--name-only", "-z"], umbrella.work_dir)
    known = {name: (branch, commit) for name, branch, commit in updated}
    lines: list[str] = []
    for name in output.split("\0"):
        if name not in known:
            continue
        branch, commit = known[name]
        lines.append(f"- {name}: {branch} to {commit[:8]}")
    message = "Update submodules to latest"
    if lines:
        message += "\n\nUpdated submodules:\n" + "\n".join(lines)
    return message + "\n"
