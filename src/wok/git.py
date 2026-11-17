"""Async subprocess runner for git and git-lfs."""

from __future__ import annotations

import asyncio
import os
import shutil
import sys
from collections.abc import Mapping, Sequence
from pathlib import Path

from wok.runtime import disable_prompt, inherit_tty
from wok.ui import section_var

_TAIL_LINES = 20


class GitError(Exception):
    """A git or git-lfs command exited non-zero."""

    def __init__(self, argv: Sequence[str], cwd: Path, tail: str, code: int) -> None:
        self.argv = list(argv)
        self.cwd = cwd
        self.tail = tail
        self.code = code
        rendered = " ".join(argv)
        message = f"`{rendered}` failed in `{cwd}`"
        if tail:
            message = f"{message}: {tail}"
        if disable_prompt.get():
            message += "\nHint: re-run with -j1 to allow interactive prompts."
        super().__init__(message)


def lfs_missing_message(work_dir: Path) -> str:
    """Fedora install hint used when a repo needs Git LFS."""
    return (
        f"Git LFS support required for `{work_dir}` but `git-lfs` is not installed.\n"
        "Install it on Fedora with:\n"
        "sudo dnf install git-lfs\n"
        "git lfs install"
    )


async def git(
    args: Sequence[str],
    cwd: Path,
    *,
    input_text: str | None = None,
    extra_env: Mapping[str, str] | None = None,
    check: bool = True,
) -> str:
    """Run git and return merged stdout/stderr."""
    code, output = await run_cmd(
        ["git", *args],
        cwd,
        input_text=input_text,
        extra_env=extra_env,
    )
    if check and code != 0:
        raise GitError(["git", *args], cwd, _tail(output), code)
    return output


async def git_lfs(args: Sequence[str], cwd: Path) -> str:
    """Run `git lfs`, with the Fedora install hint when the helper is missing."""
    argv = ["git", "lfs", *args]
    if shutil.which("git-lfs") is None:
        raise GitError(argv, cwd, lfs_missing_message(cwd), 127)
    code, output = await run_cmd(argv, cwd)
    if code != 0:
        raise GitError(argv, cwd, _tail(output), code)
    return output


async def git_raw(
    args: Sequence[str],
    cwd: Path,
    *,
    extra_env: Mapping[str, str] | None = None,
) -> tuple[int, str]:
    """Run git and return the exit code without raising."""
    return await run_cmd(["git", *args], cwd, extra_env=extra_env)


async def run_cmd(
    argv: Sequence[str],
    cwd: Path,
    *,
    input_text: str | None = None,
    extra_env: Mapping[str, str] | None = None,
) -> tuple[int, str]:
    """Run a command, streaming progress into the current section."""
    env = os.environ.copy()
    if extra_env:
        env.update({key: value for key, value in extra_env.items()})
    if disable_prompt.get():
        env["GIT_TERMINAL_PROMPT"] = "0"

    section = section_var.get()
    # -j1 keeps stdin attached so git can prompt, but stdout is always captured
    # so callers can parse refs, config, and status. Chunks are also copied to
    # the terminal when nothing else is collecting a per-repo section.
    inherit = inherit_tty.get() and input_text is None and section is None
    process = await asyncio.create_subprocess_exec(
        *argv,
        cwd=cwd,
        env=env,
        stdin=None if inherit else asyncio.subprocess.PIPE,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.STDOUT,
    )
    output, code = await _communicate(process, input_text, section, tee=inherit)
    return code, output


async def _communicate(
    process: asyncio.subprocess.Process,
    input_text: str | None,
    section: object,
    *,
    tee: bool,
) -> tuple[str, int]:
    async def feed_stdin() -> None:
        if process.stdin is None:
            return
        if input_text:
            process.stdin.write(input_text.encode())
            await process.stdin.drain()
        process.stdin.close()
        await process.stdin.wait_closed()

    writer = asyncio.create_task(feed_stdin()) if process.stdin is not None else None
    output = ""
    if process.stdout is not None:
        output = await _read_stream(process.stdout, section, tee=tee)
    if writer is not None:
        await writer
    code = await process.wait()
    return output, code if code is not None else 1


async def _read_stream(
    stream: asyncio.StreamReader, section: object, *, tee: bool
) -> str:
    chunks: list[str] = []
    while True:
        data = await stream.read(4096)
        if not data:
            break
        text = data.decode(errors="replace")
        chunks.append(text)
        feed = getattr(section, "feed", None)
        if feed is not None:
            feed(text)
        elif tee:
            sys.stderr.write(text)
            sys.stderr.flush()
    return "".join(chunks)


def _tail(output: str) -> str:
    lines = [line for line in output.splitlines() if line.strip()]
    return "\n".join(lines[-_TAIL_LINES:])
