"""Push configured repositories through git push."""

from __future__ import annotations

import asyncio
from enum import Enum
from pathlib import Path
from typing import TextIO

from wok.config import Config
from wok.config import Repo as ConfigRepo
from wok.errors import WokError
from wok.git import GitError, git
from wok.repo import (
    Repo,
    local_branch_exists,
    ls_remote_head,
    remote_exists,
    remote_name_for_branch,
    rev_parse,
)
from wok.runtime import resolve_jobs
from wok.ui import run_parallel


class PushResult(Enum):
    PUSHED = "pushed"
    UPSTREAM = "upstream"
    UP_TO_DATE = "up-to-date"
    NO_REMOTE = "no-remote"


def push(
    wok_config: Config,
    umbrella: Repo,
    stdout: TextIO,
    set_upstream: bool,
    all_repos: bool,
    branch_name: str | None,
    include_umbrella: bool,
    target_repos: list[Path],
) -> None:
    asyncio.run(
        _push(
            wok_config,
            umbrella,
            stdout,
            set_upstream,
            all_repos,
            branch_name,
            include_umbrella,
            target_repos,
        )
    )


async def _push(
    wok_config: Config,
    umbrella: Repo,
    stdout: TextIO,
    set_upstream: bool,
    all_repos: bool,
    branch_name: str | None,
    include_umbrella: bool,
    target_repos: list[Path],
) -> None:
    target_branch = branch_name or umbrella.head
    wok_config.require_known(target_repos)
    selected = _select(wok_config, umbrella, all_repos, target_repos)
    total = len(selected) + int(include_umbrella)
    if total == 0:
        stdout.write("No repositories to push\n")
        return
    stdout.write(f"Pushing {total} repositories to branch '{target_branch}'...\n")

    async def push_one(config_repo: ConfigRepo) -> PushResult:
        subrepo = umbrella.get_subrepo_by_path(config_repo.path)
        if subrepo is None:
            raise WokError(f"Cannot find submodule at `{config_repo.path}`")
        return await _push_repo(subrepo, target_branch, set_upstream)

    results = await run_parallel(
        selected,
        push_one,
        resolve_jobs(len(selected) or 1),
        label=lambda config_repo: config_repo.path.as_posix(),
    )
    for config_repo, result in zip(selected, results, strict=True):
        _write_result(stdout, config_repo.path.as_posix(), target_branch, result)

    if include_umbrella:
        try:
            umbrella_result: PushResult | BaseException = await _push_repo(
                umbrella, target_branch, set_upstream
            )
        except (GitError, WokError) as exc:
            umbrella_result = exc
        _write_result(stdout, "umbrella", target_branch, umbrella_result)

    stdout.write(f"Successfully processed {total} repositories\n")


def _select(
    wok_config: Config,
    umbrella: Repo,
    all_repos: bool,
    target_repos: list[Path],
) -> list[ConfigRepo]:
    if all_repos:
        return [
            repo
            for repo in wok_config.repos
            if not repo.is_skipped_for("push") or repo.path in target_repos
        ]
    if target_repos:
        return [repo for repo in wok_config.repos if repo.path in target_repos]
    return [repo for repo in wok_config.repos if repo.head == umbrella.head]


async def _push_repo(repo: Repo, branch: str, set_upstream: bool) -> PushResult:
    remote = await remote_name_for_branch(repo.work_dir, branch)
    if not await remote_exists(repo.work_dir, remote):
        return PushResult.NO_REMOTE
    if not await local_branch_exists(repo.work_dir, branch):
        raise WokError(f"Branch '{branch}' does not exist locally")
    local = await rev_parse(repo.work_dir, f"refs/heads/{branch}")
    try:
        remote_oid = await ls_remote_head(repo.work_dir, remote, branch)
    except GitError as exc:
        print(f"Warning: Could not check remote state: {exc}")
        remote_oid = None
        local = ""
    if remote_oid is not None and remote_oid == local:
        return PushResult.UP_TO_DATE
    await repo.lfs_push_if_needed(remote, branch)
    args = ["push"]
    if set_upstream:
        args.append("--set-upstream")
    args.extend([remote, f"{branch}:refs/heads/{branch}"])
    try:
        await git(args, repo.work_dir)
    except GitError as exc:
        text = str(exc).lower()
        if "up to date" in text or "up-to-date" in text or "non-fast-forward" in text:
            return PushResult.UP_TO_DATE
        raise
    if set_upstream:
        return PushResult.UPSTREAM
    return PushResult.PUSHED


def _write_result(
    stdout: TextIO,
    name: str,
    branch: str,
    result: PushResult | BaseException,
) -> None:
    if isinstance(result, BaseException):
        stdout.write(f"- '{name}': failed to push to '{branch}' - {result}\n")
        return
    if result is PushResult.PUSHED:
        stdout.write(f"- '{name}': pushed to '{branch}'\n")
    elif result is PushResult.UPSTREAM:
        stdout.write(f"- '{name}': pushed to '{branch}' and set upstream\n")
    elif result is PushResult.UP_TO_DATE:
        stdout.write(f"- '{name}': already up to date\n")
    else:
        stdout.write(f"- '{name}': no remote configured, skipping\n")
