"""Switch configured repos and lock the umbrella pointer."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import TextIO

from wok.config import Config
from wok.config import Repo as ConfigRepo
from wok.errors import WokError
from wok.git import GitError, git
from wok.repo import Repo, current_branch, is_worktree_clean, submodule_paths
from wok.runtime import resolve_jobs
from wok.ui import run_parallel


class SwitchResult(Enum):
    SWITCHED = "switched"
    CREATED = "created"
    ALREADY = "already"


@dataclass
class PlanItem:
    config_repo: ConfigRepo
    branch: str
    forced: bool
    preserve: bool


def switch(
    wok_config: Config,
    umbrella: Repo,
    config_path: Path,
    stdout: TextIO,
    create: bool,
    all_repos: bool,
    dirty: bool,
    branch_name: str | None,
    target_repos: list[Path],
) -> bool:
    return asyncio.run(
        _switch(
            wok_config,
            umbrella,
            config_path,
            stdout,
            create,
            all_repos,
            dirty,
            branch_name,
            target_repos,
        )
    )


async def _switch(
    wok_config: Config,
    umbrella: Repo,
    config_path: Path,
    stdout: TextIO,
    create: bool,
    all_repos: bool,
    dirty: bool,
    branch_name: str | None,
    target_repos: list[Path],
) -> bool:
    branch_name = branch_name or umbrella.head
    wok_config.require_known(target_repos)
    plan = await _plan(
        wok_config, umbrella, branch_name, all_repos, dirty, target_repos
    )
    if not plan:
        stdout.write("No repositories to switch\n")
        return False

    stdout.write(
        f"Switching {len(plan)} repositories for umbrella branch '{branch_name}'...\n"
    )
    results = await run_parallel(
        plan,
        lambda item: _switch_one(umbrella, item, create),
        resolve_jobs(len(plan)),
        label=lambda item: item.config_repo.path.as_posix(),
    )

    config_updated = False
    submodule_changed = False
    lines: list[str] = []
    for item, result in zip(plan, results, strict=True):
        name = item.config_repo.path.as_posix()
        if isinstance(result, BaseException):
            lines.append(f"- '{name}': failed to switch to '{item.branch}' - {result}")
            continue
        if item.forced:
            config_updated = (
                wok_config.set_repo_head(item.config_repo.path, branch_name)
                or config_updated
            )
        if result is SwitchResult.SWITCHED:
            lines.append(f"- '{name}': switched to '{item.branch}'")
            submodule_changed = True
        elif result is SwitchResult.CREATED:
            lines.append(f"- '{name}': created and switched to '{item.branch}'")
            submodule_changed = True
        else:
            lines.append(f"- '{name}': already on '{item.branch}'")
    stdout.write("".join(f"{line}\n" for line in lines))

    if config_updated:
        wok_config.save(config_path)

    if submodule_changed or config_updated:
        stdout.write("Locking workspace state...\n")
        await _lock_switched(umbrella, config_path, plan)
        stdout.write(f"Successfully switched and locked {len(plan)} repositories\n")
    else:
        stdout.write("No workspace changes detected; skipping lock\n")
        stdout.write(f"Successfully processed {len(plan)} repositories\n")
    return config_updated


async def _plan(
    wok_config: Config,
    umbrella: Repo,
    branch_name: str,
    all_repos: bool,
    dirty: bool,
    target_repos: list[Path],
) -> list[PlanItem]:
    plan: list[PlanItem] = []
    for config_repo in wok_config.repos:
        explicit = config_repo.path in target_repos
        if config_repo.is_skipped_for("switch") and not explicit:
            continue
        if dirty:
            has_changes = explicit or await _local_changes(umbrella, config_repo.path)
            if not has_changes:
                continue
            plan.append(PlanItem(config_repo, branch_name, True, True))
            continue
        desired = branch_name if not config_repo.head.strip() else config_repo.head
        forced = all_repos or explicit
        plan.append(
            PlanItem(
                config_repo,
                branch_name if forced else desired,
                forced,
                False,
            )
        )
    return plan


async def _local_changes(umbrella: Repo, path: Path) -> bool:
    subrepo = umbrella.get_subrepo_by_path(path)
    if subrepo is None:
        return False
    if not await is_worktree_clean(subrepo.work_dir):
        return True
    from wok.repo import pointer_oid

    recorded = await pointer_oid(umbrella.work_dir, path)
    if recorded is None:
        return False
    return recorded != await subrepo.head_commit()


async def _switch_one(umbrella: Repo, item: PlanItem, create: bool) -> SwitchResult:
    subrepo = umbrella.get_subrepo_by_path(item.config_repo.path)
    if subrepo is None:
        raise WokError(f"Cannot find submodule at `{item.config_repo.path}`")
    actual = await current_branch(subrepo.work_dir)
    if actual == item.branch:
        if not item.preserve:
            await subrepo.switch_force(item.branch)
        return SwitchResult.ALREADY
    if item.preserve:
        await subrepo.switch_or_create_preserving(item.branch, create)
        return SwitchResult.SWITCHED
    if await _branch_exists(subrepo, item.branch):
        await subrepo.switch_force(item.branch)
        return SwitchResult.SWITCHED
    if not create:
        raise WokError(
            f"Branch '{item.branch}' does not exist and --create not specified"
        )
    await git(["switch", "-c", item.branch], subrepo.work_dir)
    subrepo.head = item.branch
    return SwitchResult.CREATED


async def _branch_exists(repo: Repo, branch: str) -> bool:
    from wok.repo import local_branch_exists

    return await local_branch_exists(repo.work_dir, branch)


def _wokfile_name(config_path: Path, umbrella: Repo) -> str:
    from wok.repo import resolved

    return resolved(config_path).relative_to(resolved(umbrella.work_dir)).as_posix()


async def _lock_switched(
    umbrella: Repo, config_path: Path, plan: list[PlanItem]
) -> None:
    paths = [path.as_posix() for path in await submodule_paths(umbrella.work_dir)]
    wokfile = _wokfile_name(config_path, umbrella)
    to_add = [*paths, wokfile]
    await git(["add", "--", *to_add], umbrella.work_dir)
    output = await git(["diff", "--cached", "--name-only"], umbrella.work_dir)
    if not output.strip():
        return
    message = await _switch_message(umbrella, plan)
    try:
        await git(["commit", "-F", "-"], umbrella.work_dir, input_text=message)
    except GitError as exc:
        raise WokError(
            f"Cannot lock switched workspace in `{umbrella.work_dir}`"
        ) from exc


async def _switch_message(umbrella: Repo, plan: list[PlanItem]) -> str:
    output = await git(["diff", "--cached", "--name-only", "-z"], umbrella.work_dir)
    switched = {item.config_repo.path.as_posix() for item in plan}
    changed: list[tuple[str, str]] = []
    for name in output.split("\0"):
        if name not in switched:
            continue
        subrepo = umbrella.get_subrepo_by_path(umbrella.work_dir / name)
        if subrepo is None:
            continue
        branch = await current_branch(subrepo.work_dir)
        changed.append((name, branch))
    message = "Switch and lock submodule state"
    if changed:
        message += "\n\nSwitched submodules:"
        for name, branch in changed:
            message += f"\n- {name}: {branch}"
    return message + "\n"
