"""Git-backed repository queries and actions."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from enum import Enum
from pathlib import Path

from wok.errors import WokError
from wok.git import GitError, git, git_lfs, git_raw

_REBASE_TRUE = {"true", "interactive", "i", "merges", "m"}


class MergeResult(Enum):
    UP_TO_DATE = "up-to-date"
    FAST_FORWARD = "fast-forward"
    MERGED = "merged"
    REBASED = "rebased"
    CONFLICTS = "conflicts"


class RemoteComparison(Enum):
    UP_TO_DATE = "up-to-date"
    AHEAD = "ahead"
    BEHIND = "behind"
    DIVERGED = "diverged"
    NO_REMOTE = "no-remote"


@dataclass(frozen=True)
class Comparison:
    kind: RemoteComparison
    ahead: int = 0
    behind: int = 0


@dataclass(frozen=True)
class Tracking:
    remote: str
    remote_ref: str


class Repo:
    """A git worktree and the submodules recorded in it."""

    def __init__(self, work_dir: Path, head: str, subrepos: list[Repo]) -> None:
        self.work_dir = work_dir
        self.head = head
        self.subrepos = subrepos

    @classmethod
    def open(cls, work_dir: Path, head_name: str | None = None) -> Repo:
        return asyncio.run(cls.open_async(work_dir, head_name))

    @classmethod
    async def open_async(cls, work_dir: Path, head_name: str | None = None) -> Repo:
        work_dir = resolved(work_dir)
        head = head_name if head_name is not None else await current_branch(work_dir)
        subrepos: list[Repo] = []
        for path in await submodule_paths(work_dir):
            subrepos.append(await cls.open_async(work_dir / path))
        return cls(work_dir, head, subrepos)

    def get_subrepo_by_path(self, subrepo_path: Path) -> Repo | None:
        target = (
            subrepo_path if subrepo_path.is_absolute() else self.work_dir / subrepo_path
        )
        resolved = target.resolve()
        for subrepo in self.subrepos:
            if subrepo.work_dir == resolved:
                return subrepo
        return None

    async def switch(self, branch: str) -> None:
        await git(["switch", branch], self.work_dir)
        self.head = await current_branch(self.work_dir)

    async def switch_force(self, branch: str) -> None:
        await git(["switch", "--discard-changes", branch], self.work_dir)
        self.head = await current_branch(self.work_dir)

    async def checkout_path_from_head(self, path: Path) -> None:
        await git(["checkout", "HEAD", "--", path.as_posix()], self.work_dir)

    async def switch_or_create_preserving(self, branch: str, create: bool) -> None:
        actual = await current_branch(self.work_dir)
        if actual == branch:
            return
        if await local_branch_exists(self.work_dir, branch):
            await git(["symbolic-ref", "HEAD", f"refs/heads/{branch}"], self.work_dir)
            self.head = branch
            return
        if create:
            await git(["branch", branch], self.work_dir)
            await git(["symbolic-ref", "HEAD", f"refs/heads/{branch}"], self.work_dir)
            self.head = branch
            return
        raise WokError(f"Branch '{branch}' does not exist and --create not specified")

    async def ensure_on_branch(self, branch: str) -> None:
        await self._ensure(branch, create_at_head=True)

    async def ensure_on_branch_existing_or_remote(
        self, branch: str, create: bool
    ) -> None:
        await self._ensure(branch, create_at_head=create)

    async def _ensure(self, branch: str, *, create_at_head: bool) -> None:
        if not await is_worktree_clean(self.work_dir):
            raise WokError(
                "Refusing to switch branches with uncommitted changes "
                f"in `{self.work_dir}`"
            )
        actual = await current_branch(self.work_dir)
        if actual == branch:
            return
        if await local_branch_exists(self.work_dir, branch):
            await self.switch_force(branch)
            return
        remote_name = await remote_name_for_branch(self.work_dir, branch)
        if await remote_exists(self.work_dir, remote_name):
            await git(["fetch", "--progress", remote_name], self.work_dir)
        if await remote_branch_exists(self.work_dir, remote_name, branch):
            await git(
                ["switch", "-c", branch, "--track", f"{remote_name}/{branch}"],
                self.work_dir,
            )
            self.head = branch
            return
        if create_at_head:
            await git(["switch", "-c", branch], self.work_dir)
            self.head = branch
            return
        raise WokError(f"Branch '{branch}' does not exist and --create not specified")

    async def fetch(self) -> None:
        if self.head == "<detached>":
            return
        tracking = await tracking_branch(self.work_dir, self.head)
        if tracking is None or not await remote_exists(self.work_dir, tracking.remote):
            return
        try:
            await git(["fetch", "--progress", tracking.remote], self.work_dir)
        except GitError as exc:
            raise WokError(
                f"Failed to fetch from remote '{tracking.remote}' "
                f"for repo at `{self.work_dir}`"
            ) from exc

    async def pull(self, branch: str) -> MergeResult:
        tracking = await tracking_branch(self.work_dir, branch)
        if tracking is None or not await remote_exists(self.work_dir, tracking.remote):
            return MergeResult.UP_TO_DATE
        before = await rev_parse(self.work_dir, "HEAD")
        if await pull_rebases(self.work_dir, branch):
            pull_args = ["pull", "--rebase", "--no-edit", "--progress"]
        elif await config_get(self.work_dir, "pull.ff") == "only":
            pull_args = ["pull", "--ff-only", "--no-edit", "--progress"]
        else:
            pull_args = ["pull", "--no-rebase", "--no-edit", "--progress"]
        try:
            await git(pull_args, self.work_dir)
        except GitError as exc:
            if await has_conflict(self.work_dir):
                return MergeResult.CONFLICTS
            raise WokError(f"Cannot pull `{self.work_dir}`") from exc
        if await has_conflict(self.work_dir):
            return MergeResult.CONFLICTS
        after = await rev_parse(self.work_dir, "HEAD")
        if before == after:
            return MergeResult.UP_TO_DATE
        remote_tip = (await rev_parse(self.work_dir, tracking.remote_ref)).strip()
        if after.strip() == remote_tip and await is_ancestor(
            self.work_dir, before, after
        ):
            await self.lfs_pull_if_needed()
            return MergeResult.FAST_FORWARD
        if await parent_count(self.work_dir) > 1:
            await self.lfs_pull_if_needed()
            return MergeResult.MERGED
        if await pull_rebases(self.work_dir, branch):
            await self.lfs_pull_if_needed()
            return MergeResult.REBASED
        await self.lfs_pull_if_needed()
        return MergeResult.MERGED

    async def comparison(self, branch: str) -> Comparison | None:
        tracking = await tracking_branch(self.work_dir, branch)
        if tracking is None:
            return None
        code, _output = await git_raw(
            ["rev-parse", "--verify", "--quiet", tracking.remote_ref],
            self.work_dir,
        )
        if code != 0:
            return Comparison(RemoteComparison.NO_REMOTE)
        output = await git(
            ["rev-list", "--left-right", "--count", f"HEAD...{tracking.remote_ref}"],
            self.work_dir,
        )
        ahead_text, behind_text = output.split()
        ahead, behind = int(ahead_text), int(behind_text)
        if ahead and behind:
            return Comparison(RemoteComparison.DIVERGED, ahead, behind)
        if ahead:
            return Comparison(RemoteComparison.AHEAD, ahead, behind)
        if behind:
            return Comparison(RemoteComparison.BEHIND, ahead, behind)
        return Comparison(RemoteComparison.UP_TO_DATE)

    async def uses_lfs(self) -> bool:
        attributes = self.work_dir / ".gitattributes"
        if attributes.exists():
            for line in attributes.read_text().splitlines():
                trimmed = line.strip()
                if trimmed and not trimmed.startswith("#") and "filter=lfs" in trimmed:
                    return True
        if (self.work_dir / ".lfsconfig").exists():
            return True
        git_dir = await git_directory(self.work_dir)
        return (git_dir / "lfs").exists()

    async def lfs_pull_if_needed(self) -> None:
        if await self.uses_lfs():
            await git_lfs(["pull"], self.work_dir)

    async def lfs_push_if_needed(self, remote: str, branch: str) -> None:
        if await self.uses_lfs():
            await git_lfs(["push", remote, branch], self.work_dir)

    async def head_commit(self) -> str:
        return (await rev_parse(self.work_dir, "HEAD")).strip()

    async def commit_message_subject(self) -> str:
        output = await git(["log", "-1", "--format=%s"], self.work_dir)
        return output.strip()


def resolved(path: Path) -> Path:
    """Return an absolute path without calling pathlib from an async function."""
    return path.resolve()


async def init_configured_submodules(
    work_dir: Path, configured_paths: list[Path]
) -> None:
    """Initialize configured submodules, shallowest paths first."""
    ordered = sorted(configured_paths, key=lambda path: len(path.parts))
    for configured in ordered:
        await _init_path(work_dir, configured)


async def _init_path(work_dir: Path, configured: Path) -> None:
    if not configured.parts:
        return
    first = configured.parts[0]
    if not await _is_submodule(work_dir, first):
        return
    await git(["submodule", "update", "--init", "--", first], work_dir)
    rest = Path(*configured.parts[1:])
    if rest.parts:
        await _init_path(work_dir / first, rest)


async def _is_submodule(work_dir: Path, name: str) -> bool:
    if not (work_dir / ".gitmodules").exists():
        return False
    code, output = await git_raw(
        ["config", "-f", ".gitmodules", "--get-regexp", r"^submodule\..*\.path$"],
        work_dir,
    )
    if code != 0:
        return False
    return any(
        line.split(maxsplit=1)[-1] == name
        for line in output.splitlines()
        if line.strip()
    )


async def submodule_paths(work_dir: Path) -> list[Path]:
    if not (work_dir / ".gitmodules").exists():
        return []
    code, output = await git_raw(
        ["config", "-f", ".gitmodules", "--get-regexp", r"^submodule\..*\.path$"],
        work_dir,
    )
    if code != 0:
        return []
    paths: list[Path] = []
    for line in output.splitlines():
        if not line.strip():
            continue
        paths.append(Path(line.split(maxsplit=1)[1]))
    return paths


async def current_branch(work_dir: Path) -> str:
    code, output = await git_raw(["symbolic-ref", "--short", "-q", "HEAD"], work_dir)
    if code != 0:
        return "<detached>"
    return output.strip()


async def local_branch_exists(work_dir: Path, branch: str) -> bool:
    code, _output = await git_raw(
        ["show-ref", "--verify", "--quiet", f"refs/heads/{branch}"], work_dir
    )
    return code == 0


async def remote_exists(work_dir: Path, remote: str) -> bool:
    code, _output = await git_raw(["remote", "get-url", remote], work_dir)
    return code == 0


async def remote_branch_exists(work_dir: Path, remote: str, branch: str) -> bool:
    code, _output = await git_raw(
        ["rev-parse", "--verify", "--quiet", f"refs/remotes/{remote}/{branch}"],
        work_dir,
    )
    return code == 0


async def remote_name_for_branch(work_dir: Path, branch: str) -> str:
    tracking = await tracking_branch(work_dir, branch)
    if tracking is None:
        return "origin"
    return tracking.remote


async def tracking_branch(work_dir: Path, branch: str) -> Tracking | None:
    remote = await config_get(work_dir, f"branch.{branch}.remote")
    merge = await config_get(work_dir, f"branch.{branch}.merge")
    if not remote or not merge:
        return None
    short = merge.removeprefix("refs/heads/")
    return Tracking(remote=remote, remote_ref=f"refs/remotes/{remote}/{short}")


async def config_get(work_dir: Path, key: str) -> str | None:
    code, output = await git_raw(["config", "--get", key], work_dir)
    if code != 0:
        return None
    text = output.strip()
    return text or None


async def rev_parse(work_dir: Path, revision: str) -> str:
    output = await git(["rev-parse", revision], work_dir)
    return output.strip()


def _git_dir_path(work_dir: Path, output: str) -> Path:
    path = Path(output.strip())
    if not path.is_absolute():
        path = work_dir / path
    return path.resolve()


async def git_directory(work_dir: Path) -> Path:
    output = await git(["rev-parse", "--git-dir"], work_dir)
    return _git_dir_path(work_dir, output)


async def is_worktree_clean(work_dir: Path) -> bool:
    return not await status_entries(work_dir)


async def status_entries(work_dir: Path) -> list[tuple[str, str]]:
    _code, output = await git_raw(
        ["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        work_dir,
    )
    return _parse_porcelain_z(output)


def _parse_porcelain_z(output: str) -> list[tuple[str, str]]:
    if not output:
        return []
    parts = output.split("\0")
    entries: list[tuple[str, str]] = []
    index = 0
    while index < len(parts):
        record = parts[index]
        if record == "":
            index += 1
            continue
        if len(record) < 3:
            index += 1
            continue
        xy = record[:2]
        path = record[3:]
        if "R" in xy or "C" in xy:
            index += 1
            if index < len(parts) and parts[index]:
                path = parts[index]
        entries.append((xy, path))
        index += 1
    return entries


async def has_conflict(work_dir: Path) -> bool:
    git_dir = await git_directory(work_dir)
    if (git_dir / "MERGE_HEAD").exists():
        return True
    if (git_dir / "rebase-merge").exists() or (git_dir / "rebase-apply").exists():
        return True
    code, output = await git_raw(["diff", "--name-only", "--diff-filter=U"], work_dir)
    return code == 0 and bool(output.strip())


async def is_ancestor(work_dir: Path, ancestor: str, descendant: str) -> bool:
    code, _output = await git_raw(
        ["merge-base", "--is-ancestor", ancestor.strip(), descendant.strip()],
        work_dir,
    )
    return code == 0


async def parent_count(work_dir: Path) -> int:
    output = await git(["rev-list", "--parents", "-n", "1", "HEAD"], work_dir)
    return max(0, len(output.split()) - 1)


async def pull_rebases(work_dir: Path, branch: str) -> bool:
    for key in (f"branch.{branch}.rebase", "pull.rebase"):
        value = await config_get(work_dir, key)
        if value is None:
            continue
        return value.lower() in _REBASE_TRUE
    return False


async def pointer_oid(work_dir: Path, path: Path) -> str | None:
    code, output = await git_raw(
        ["ls-files", "--stage", "--", path.as_posix()], work_dir
    )
    if code == 0:
        for line in output.splitlines():
            parts = line.split()
            if len(parts) >= 2 and parts[0] == "160000":
                return parts[1]
    code, output = await git_raw(["rev-parse", f"HEAD:{path.as_posix()}"], work_dir)
    if code == 0 and output.strip():
        return output.strip()
    return None


async def ls_remote_head(work_dir: Path, remote: str, branch: str) -> str | None:
    code, output = await git_raw(
        ["ls-remote", "--heads", remote, f"refs/heads/{branch}"], work_dir
    )
    if code != 0 or not output.strip():
        return None
    return output.split()[0]


async def list_tags(work_dir: Path) -> list[tuple[str, int | None]]:
    code, output = await git_raw(
        [
            "for-each-ref",
            "--format=%(refname:short)%09%(creatordate:unix)",
            "refs/tags",
        ],
        work_dir,
    )
    if code != 0:
        raise WokError(f"Cannot list tags in `{work_dir}`")
    tags: list[tuple[str, int | None]] = []
    for line in output.splitlines():
        if not line.strip():
            continue
        name, _, stamp = line.partition("\t")
        tags.append((name, int(stamp) if stamp else None))
    tags.sort(key=lambda item: (-(item[1] or -1), item[0]))
    return tags


async def head_has_tags(work_dir: Path) -> bool:
    code, output = await git_raw(["tag", "--points-at", "HEAD"], work_dir)
    return code == 0 and bool(output.strip())


async def describe_head(work_dir: Path) -> tuple[str, int] | None:
    code, output = await git_raw(["describe", "--tags"], work_dir)
    if code != 0 or not output.strip():
        return None
    return _parse_describe(output.strip())


def _parse_describe(output: str) -> tuple[str, int]:
    parts = output.rsplit("-", 2)
    if len(parts) == 3 and parts[2].startswith("g") and parts[1].isdigit():
        return parts[0], int(parts[1])
    return output, 0


def truncate_subject(message: str) -> str:
    first = message.splitlines()[0] if message else "(no message)"
    if len(first) > 50:
        return first[:47] + "..."
    return first
