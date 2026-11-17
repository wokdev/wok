"""Workspace status lines."""

from __future__ import annotations

import asyncio
from pathlib import Path
from typing import TextIO

from wok.config import Config
from wok.repo import Comparison, RemoteComparison, Repo, status_entries
from wok.runtime import resolve_jobs
from wok.ui import run_parallel

_WT_NEW = "??"
_INDEX_NEW = "A "


def status(wok_config: Config, umbrella: Repo, stdout: TextIO, fetch: bool) -> None:
    asyncio.run(_status(wok_config, umbrella, stdout, fetch))


async def _status(
    wok_config: Config, umbrella: Repo, stdout: TextIO, fetch: bool
) -> None:
    if fetch:
        targets: list[Repo] = [umbrella]
        for config_repo in wok_config.repos:
            subrepo = umbrella.get_subrepo_by_path(config_repo.path)
            if subrepo is not None:
                targets.append(subrepo)
        results = await run_parallel(
            targets,
            lambda repo: repo.fetch(),
            resolve_jobs(len(targets)),
            label=lambda repo: "umbrella" if repo is umbrella else repo.work_dir.name,
        )
        for result in results:
            if isinstance(result, BaseException):
                raise result

    rows: list[str] = []
    umbrella_label = await _umbrella_label(umbrella, wok_config)
    rows.append(
        _row(
            "✅" if umbrella_label == "clean" else _symbol(umbrella_label),
            "umbrella",
            umbrella.head,
            umbrella_label,
        )
    )

    for config_repo in wok_config.repos:
        subrepo = umbrella.get_subrepo_by_path(config_repo.path)
        if subrepo is None:
            continue
        clean = await _is_clean(subrepo.work_dir, None)
        new_commits = await _ahead_of_pointer(umbrella, config_repo.path, subrepo)
        if not clean:
            label = "has uncommitted changes"
        elif new_commits:
            label = "new commits"
        else:
            label = "clean"
        rows.append(
            _row(_symbol(label), config_repo.path.as_posix(), subrepo.head, label)
        )

    stdout.write("".join(f"{row}\n" for row in rows))


async def _umbrella_label(umbrella: Repo, wok_config: Config) -> str:
    entries = await _relevant(umbrella.work_dir, wok_config)
    if not entries:
        label = "clean"
    elif all(_is_configured_path(path, wok_config) for _xy, path in entries):
        label = "needs locking"
    else:
        label = "has uncommitted changes"
    if label == "clean":
        comparison = await umbrella.comparison(umbrella.head)
        if comparison is not None and _has_local_commits(comparison):
            return "new commits"
    return label


def _has_local_commits(comparison: Comparison) -> bool:
    return (
        comparison.kind in {RemoteComparison.AHEAD, RemoteComparison.DIVERGED}
        and comparison.ahead > 0
    )


async def _is_clean(work_dir: Path, wok_config: Config | None) -> bool:
    return not await _relevant(work_dir, wok_config)


async def _relevant(work_dir: Path, wok_config: Config | None) -> list[tuple[str, str]]:
    relevant: list[tuple[str, str]] = []
    for xy, path in await status_entries(work_dir):
        if xy == _WT_NEW and path == "wok.toml":
            continue
        if xy == _INDEX_NEW and path == ".gitmodules":
            continue
        if (
            xy == _INDEX_NEW
            and wok_config is not None
            and _is_configured_path(path, wok_config)
        ):
            continue
        relevant.append((xy, path))
    return relevant


def _is_configured_path(path: str, wok_config: Config) -> bool:
    return any(repo.path.as_posix() == path for repo in wok_config.repos)


async def _ahead_of_pointer(umbrella: Repo, path: Path, subrepo: Repo) -> bool:
    from wok.repo import pointer_oid

    recorded = await pointer_oid(umbrella.work_dir, path)
    if recorded is None:
        return False
    head = await subrepo.head_commit()
    return recorded != head


def _symbol(label: str) -> str:
    return {
        "clean": "✅",
        "needs locking": "🔒",
        "new commits": "⬆",
        "has uncommitted changes": "❌",
    }[label]


def _row(symbol: str, name: str, branch: str, label: str) -> str:
    return f"{symbol} {name} [{branch}]: {label}"
