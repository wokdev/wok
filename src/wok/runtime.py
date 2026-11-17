"""Process-wide options shared by the CLI and git runner."""

from __future__ import annotations

import os
from contextvars import ContextVar
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Runtime:
    """Global flags for one wok invocation."""

    jobs: int | None = None
    verbose: bool = False
    wokfile: Path = Path("wok.toml")


runtime_var: ContextVar[Runtime | None] = ContextVar("wok_runtime", default=None)


def current_runtime() -> Runtime:
    """Return the active runtime, or the defaults when the CLI has not set one."""
    return runtime_var.get() or Runtime()


inherit_tty: ContextVar[bool] = ContextVar("wok_inherit_tty", default=False)
disable_prompt: ContextVar[bool] = ContextVar("wok_disable_prompt", default=False)


def resolve_jobs(target_count: int, *, force: int | None = None) -> int:
    """Pick a job count from an override, -j, WOK_JOBS, or the target count."""
    if force is not None:
        return max(1, force)
    requested = current_runtime().jobs
    if requested is None:
        env = os.environ.get("WOK_JOBS")
        if env:
            requested = int(env)
    if requested is None:
        return max(1, min(8, target_count or 1))
    return max(1, requested)
