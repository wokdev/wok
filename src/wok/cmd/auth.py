"""Check that git can talk to the current repository's remote."""

from __future__ import annotations

import asyncio
from typing import TextIO

from wok.git import GitError, git
from wok.repo import Repo, remote_exists, tracking_branch
from wok.runtime import current_runtime


def check_auth(repo: Repo, stdout: TextIO, *, verbose: bool | None = None) -> None:
    asyncio.run(
        _test_auth(
            repo, stdout, verbose if verbose is not None else current_runtime().verbose
        )
    )


async def _test_auth(repo: Repo, stdout: TextIO, verbose: bool) -> None:
    stdout.write("Testing authentication for repository...\n")
    stdout.write(f"Repository: {repo.work_dir}\n\n")
    branch = repo.head
    if branch == "<detached>":
        raise GitError(
            ["git", "symbolic-ref", "HEAD"], repo.work_dir, "Cannot get branch name", 1
        )
    stdout.write(f"Current branch: {branch}\n")
    tracking = await tracking_branch(repo.work_dir, branch)
    if tracking is None:
        stdout.write(f"No tracking branch configured for '{branch}'\n\n")
        stdout.write("To set up tracking, run:\n")
        stdout.write(f"  git branch --set-upstream-to=<remote>/<branch> {branch}\n")
        return
    stdout.write(f"Remote: {tracking.remote}\n")
    stdout.write(f"Remote ref: {tracking.remote_ref}\n\n")
    if not await remote_exists(repo.work_dir, tracking.remote):
        raise GitError(
            ["git", "remote"],
            repo.work_dir,
            f"Remote '{tracking.remote}' not found",
            1,
        )
    stdout.write("Attempting to connect to remote...\n\n")
    extra_env = {"GIT_TRACE": "1", "GIT_SSH_COMMAND": "ssh -v"} if verbose else None
    try:
        output = await git(
            ["ls-remote", "--heads", tracking.remote],
            repo.work_dir,
            extra_env=extra_env,
        )
    except GitError as exc:
        stdout.write(f"\n? Connection failed: {exc}\n\n")
        stdout.write("Troubleshooting steps:\n")
        stdout.write("1. Check SSH agent is running: ssh-add -l\n")
        stdout.write("2. Verify SSH_AUTH_SOCK is set: echo $SSH_AUTH_SOCK\n")
        stdout.write("3. Test SSH connection: ssh -T git@<hostname>\n")
        stdout.write("4. Check SSH keys exist: ls -la ~/.ssh/\n")
        raise
    stdout.write("\n? Connection successful!\n\n")
    stdout.write("Available remote heads:\n")
    for line in output.splitlines():
        if not line.strip():
            continue
        ref = line.split()[-1]
        stdout.write(f"  - {ref}\n")
