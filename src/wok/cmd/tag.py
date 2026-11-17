"""Create, list, and push tags through git."""

from __future__ import annotations

import asyncio
from enum import Enum
from pathlib import Path
from typing import TextIO

from wok.config import Config
from wok.config import Repo as ConfigRepo
from wok.errors import WokError
from wok.git import GitError, git, git_raw
from wok.repo import (
    Repo,
    describe_head,
    head_has_tags,
    list_tags,
    local_branch_exists,
    remote_exists,
    remote_name_for_branch,
)
from wok.runtime import resolve_jobs
from wok.ui import run_parallel


class TagResult(Enum):
    CREATED = "created"
    EXISTS = "exists"


class TagPushResult(Enum):
    PUSHED = "pushed"
    SKIPPED = "skipped"


def tag(
    wok_config: Config,
    umbrella: Repo,
    stdout: TextIO,
    tag_name: str | None,
    sign: bool,
    message: str | None,
    push: bool,
    all_repos: bool,
    include_umbrella: bool,
    target_repos: list[Path],
) -> None:
    if tag_name is None:
        tag_list(
            wok_config, umbrella, stdout, all_repos, include_umbrella, target_repos
        )
        if push:
            tag_push(
                wok_config, umbrella, stdout, all_repos, include_umbrella, target_repos
            )
        return
    tag_create(
        wok_config,
        umbrella,
        stdout,
        tag_name,
        sign,
        message,
        all_repos,
        include_umbrella,
        False,
        target_repos,
    )
    if push:
        tag_push(
            wok_config, umbrella, stdout, all_repos, include_umbrella, target_repos
        )


def tag_list(
    wok_config: Config,
    umbrella: Repo,
    stdout: TextIO,
    all_repos: bool,
    include_umbrella: bool,
    target_repos: list[Path],
) -> None:
    asyncio.run(
        _tag_list(
            wok_config, umbrella, stdout, all_repos, include_umbrella, target_repos
        )
    )


def tag_create(
    wok_config: Config,
    umbrella: Repo,
    stdout: TextIO,
    tag_name: str,
    sign: bool,
    message: str | None,
    all_repos: bool,
    include_umbrella: bool,
    updated: bool,
    target_repos: list[Path],
) -> None:
    asyncio.run(
        _tag_create(
            wok_config,
            umbrella,
            stdout,
            tag_name,
            sign,
            message,
            all_repos,
            include_umbrella,
            updated,
            target_repos,
        )
    )


def tag_push(
    wok_config: Config,
    umbrella: Repo,
    stdout: TextIO,
    all_repos: bool,
    include_umbrella: bool,
    target_repos: list[Path],
) -> None:
    asyncio.run(
        _tag_push(
            wok_config, umbrella, stdout, all_repos, include_umbrella, target_repos
        )
    )


def _select(
    wok_config: Config,
    umbrella: Repo,
    all_repos: bool,
    target_repos: list[Path],
) -> list[ConfigRepo]:
    wok_config.require_known(target_repos)
    if all_repos:
        return [
            repo
            for repo in wok_config.repos
            if not repo.is_skipped_for("tag") or repo.path in target_repos
        ]
    if target_repos:
        return [repo for repo in wok_config.repos if repo.path in target_repos]
    return [
        repo
        for repo in wok_config.repos
        if repo.head == umbrella.head and not repo.is_skipped_for("tag")
    ]


async def _tag_list(
    wok_config: Config,
    umbrella: Repo,
    stdout: TextIO,
    all_repos: bool,
    include_umbrella: bool,
    target_repos: list[Path],
) -> None:
    selected = _select(wok_config, umbrella, all_repos, target_repos)
    if len(selected) + int(include_umbrella) == 0:
        stdout.write("No repositories to tag\n")
        return
    if include_umbrella:
        stdout.write(await _list_line(umbrella, "umbrella") + "\n")
    for config_repo in selected:
        subrepo = umbrella.get_subrepo_by_path(config_repo.path)
        name = config_repo.path.as_posix()
        if subrepo is None:
            stdout.write(f"❌ {name}: failed to list tags - missing submodule\n")
            continue
        stdout.write(await _list_line(subrepo, name) + "\n")


async def _list_line(repo: Repo, name: str) -> str:
    try:
        tags = await list_tags(repo.work_dir)
        described = await describe_head(repo.work_dir)
    except (GitError, WokError) as exc:
        return f"❌ {name}: failed to list tags - {exc}"
    if described is None:
        symbol, head_text = "✅", "no-tags"
    elif described[1] > 0:
        symbol, head_text = "⬆", f"{described[0]}-{described[1]}"
    else:
        symbol, head_text = "✅", described[0]
    names = [tag_name for tag_name, _stamp in tags]
    if not names:
        return f"{symbol} {name} [{head_text}]: no tags found"
    return f"{symbol} {name} [{head_text}]: {', '.join(names)}"


async def _tag_create(
    wok_config: Config,
    umbrella: Repo,
    stdout: TextIO,
    tag_name: str,
    sign: bool,
    message: str | None,
    all_repos: bool,
    include_umbrella: bool,
    updated: bool,
    target_repos: list[Path],
) -> None:
    selected = _select(wok_config, umbrella, all_repos, target_repos)
    if updated:
        filtered: list[ConfigRepo] = []
        for config_repo in selected:
            subrepo = umbrella.get_subrepo_by_path(config_repo.path)
            if subrepo is not None and not await head_has_tags(subrepo.work_dir):
                filtered.append(config_repo)
        selected = filtered
    total = len(selected) + int(include_umbrella)
    if total == 0:
        stdout.write("No repositories to tag\n")
        return
    stdout.write(f"Creating tag '{tag_name}' in {total} repositories...\n")

    async def create_one(config_repo: ConfigRepo) -> TagResult:
        subrepo = umbrella.get_subrepo_by_path(config_repo.path)
        if subrepo is None:
            raise WokError(f"Cannot find submodule at `{config_repo.path}`")
        return await _create_tag(subrepo, tag_name, sign, message)

    jobs = 1 if sign else resolve_jobs(max(1, len(selected) + int(include_umbrella)))
    if include_umbrella:
        try:
            result: TagResult | BaseException = await _create_tag(
                umbrella, tag_name, sign, message
            )
        except (GitError, WokError) as exc:
            result = exc
        _write_create(stdout, "umbrella", tag_name, result)
    results = await run_parallel(
        selected,
        create_one,
        jobs,
        label=lambda config_repo: config_repo.path.as_posix(),
    )
    for config_repo, result in zip(selected, results, strict=True):
        _write_create(stdout, config_repo.path.as_posix(), tag_name, result)
    stdout.write(f"Successfully processed {total} repositories\n")


def _write_create(
    stdout: TextIO, name: str, tag_name: str, result: TagResult | BaseException
) -> None:
    if isinstance(result, BaseException):
        stdout.write(f"- '{name}': failed to create tag '{tag_name}' - {result}\n")
    elif result is TagResult.CREATED:
        stdout.write(f"- '{name}': created tag '{tag_name}'\n")
    else:
        stdout.write(f"- '{name}': tag '{tag_name}' already exists\n")


async def _create_tag(
    repo: Repo, tag_name: str, sign: bool, message: str | None
) -> TagResult:
    code, _output = await git_raw(
        ["rev-parse", "--verify", "--quiet", f"refs/tags/{tag_name}"], repo.work_dir
    )
    if code == 0:
        return TagResult.EXISTS
    if sign or message is not None:
        tag_message = message if message is not None else f"Tag {tag_name}"
        args = ["tag", "-s" if sign else "-a", "-m", tag_message, tag_name]
        await git(args, repo.work_dir)
    else:
        await git(["tag", tag_name], repo.work_dir)
    return TagResult.CREATED


async def _tag_push(
    wok_config: Config,
    umbrella: Repo,
    stdout: TextIO,
    all_repos: bool,
    include_umbrella: bool,
    target_repos: list[Path],
) -> None:
    selected = _select(wok_config, umbrella, all_repos, target_repos)
    total = len(selected) + int(include_umbrella)
    if total == 0:
        stdout.write("No repositories to tag\n")
        return
    stdout.write("Pushing tags to remotes...\n")

    async def push_one(config_repo: ConfigRepo) -> TagPushResult:
        subrepo = umbrella.get_subrepo_by_path(config_repo.path)
        if subrepo is None:
            raise WokError(f"Cannot find submodule at `{config_repo.path}`")
        return await _push_tags(subrepo)

    if include_umbrella:
        try:
            result: TagPushResult | BaseException = await _push_tags(umbrella)
        except (GitError, WokError) as exc:
            result = exc
        _write_push(stdout, "umbrella", result)
    results = await run_parallel(
        selected,
        push_one,
        resolve_jobs(max(1, len(selected))),
        label=lambda config_repo: config_repo.path.as_posix(),
    )
    for config_repo, result in zip(selected, results, strict=True):
        _write_push(stdout, config_repo.path.as_posix(), result)
    stdout.write(f"Successfully processed {total} repositories\n")


def _write_push(
    stdout: TextIO, name: str, result: TagPushResult | BaseException
) -> None:
    if isinstance(result, BaseException):
        stdout.write(f"- '{name}': failed to push tags - {result}\n")
    elif result is TagPushResult.PUSHED:
        stdout.write(f"- '{name}': pushed tags\n")
    else:
        stdout.write(f"- '{name}': no tags to push\n")


async def _push_tags(repo: Repo) -> TagPushResult:
    branch = repo.head if repo.head != "<detached>" else await _branch_or_error(repo)
    if (
        not await local_branch_exists(repo.work_dir, branch)
        and repo.head == "<detached>"
    ):
        raise WokError(f"Cannot get branch name for repo at `{repo.work_dir}`")
    remote = await remote_name_for_branch(repo.work_dir, branch)
    if not await remote_exists(repo.work_dir, remote):
        raise WokError(f"No remote '{remote}' configured")
    tags = await list_tags(repo.work_dir)
    if not tags:
        return TagPushResult.SKIPPED
    code, output = await git_raw(["ls-remote", "--tags", remote], repo.work_dir)
    remote_tags: dict[str, str] = {}
    if code == 0:
        for line in output.splitlines():
            if not line.strip():
                continue
            oid, ref = line.split(maxsplit=1)
            if ref.startswith("refs/tags/") and not ref.endswith("^{}"):
                remote_tags[ref] = oid
    refspecs: list[str] = []
    for name, _stamp in tags:
        refname = f"refs/tags/{name}"
        local_oid = (await git(["rev-parse", refname], repo.work_dir)).strip()
        if remote_tags.get(refname) == local_oid:
            continue
        refspecs.append(f"{refname}:{refname}")
    if not refspecs:
        return TagPushResult.SKIPPED
    await git(["push", remote, *refspecs], repo.work_dir)
    return TagPushResult.PUSHED


async def _branch_or_error(repo: Repo) -> str:
    from wok.repo import current_branch

    branch = await current_branch(repo.work_dir)
    if branch == "<detached>":
        raise WokError(f"Cannot get branch name for repo at `{repo.work_dir}`")
    return branch
