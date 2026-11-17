"""Turn a directory of repositories into a wok workspace."""

from __future__ import annotations

import asyncio
from pathlib import Path
from typing import TextIO

from wok.cmd.init import init
from wok.errors import WokError
from wok.git import GitError, git, git_raw
from wok.repo import Repo


def assemble(workspace_dir: Path, config_path: Path, stdout: TextIO) -> None:
    if not workspace_dir.exists():
        raise WokError(f"Workspace directory `{workspace_dir}` does not exist")
    if not workspace_dir.is_dir():
        raise WokError(f"Workspace path `{workspace_dir}` is not a directory")
    asyncio.run(_assemble(workspace_dir, config_path, stdout))


def _child_directories(workspace_dir: Path) -> list[Path]:
    skipped = {".git", ".gitmodules"}
    children = [
        entry
        for entry in workspace_dir.iterdir()
        if entry.is_dir() and entry.name not in skipped
    ]
    return sorted(children, key=lambda item: item.name)


def _resolved(path: Path) -> Path:
    return path.resolve()


async def _assemble(workspace_dir: Path, config_path: Path, stdout: TextIO) -> None:
    stdout.write(f"Assembling workspace in `{workspace_dir}`\n")
    await _ensure_repo(workspace_dir)
    known = {path.as_posix() for path in await _submodule_paths(workspace_dir)}
    for entry in _child_directories(workspace_dir):
        relative = entry.relative_to(workspace_dir)
        await _ensure_repo(entry)
        if relative.as_posix() in known:
            continue
        source = _resolved(entry)
        try:
            await git(
                [
                    "-c",
                    "protocol.file.allow=always",
                    "submodule",
                    "add",
                    str(source),
                    relative.as_posix(),
                ],
                workspace_dir,
            )
            await git(
                ["submodule", "absorbgitdirs", relative.as_posix()], workspace_dir
            )
        except GitError as exc:
            raise WokError(f"Cannot add `{relative}` as submodule") from exc
        origin = await _origin_url(entry)
        if origin:
            key = f"submodule.{relative.as_posix()}.url"
            await git(["config", "-f", ".gitmodules", key, origin], workspace_dir)
            await git(["config", key, origin], workspace_dir)
        stdout.write(f"Registered `{relative.as_posix()}` as submodule\n")
        known.add(relative.as_posix())

    umbrella = await Repo.open_async(workspace_dir)
    init(config_path, umbrella, stdout)


async def _ensure_repo(path: Path) -> None:
    code, output = await git_raw(["rev-parse", "--show-toplevel"], path)
    toplevel = _resolved(Path(output.strip())) if code == 0 and output.strip() else None
    if toplevel != _resolved(path):
        await git(["init", "-b", "main"], path)
    code, _output = await git_raw(["rev-parse", "--verify", "HEAD"], path)
    if code != 0:
        await git(
            [
                "-c",
                "user.name=git-wok assemble",
                "-c",
                "user.email=assemble@git-wok.dev",
                "commit",
                "--allow-empty",
                "-m",
                "Initial commit (git-wok assemble)",
            ],
            path,
        )


async def _submodule_paths(work_dir: Path) -> list[Path]:
    from wok.repo import submodule_paths

    return await submodule_paths(work_dir)


async def _origin_url(path: Path) -> str | None:
    code, output = await git_raw(["remote", "get-url", "origin"], path)
    if code != 0:
        return None
    return output.strip() or None
