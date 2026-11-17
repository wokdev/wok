"""Wok errors and the stderr format that replaces anyhow."""

from __future__ import annotations


class WokError(Exception):
    """A user-facing wok failure."""


def format_error(exc: BaseException) -> str:
    """Render `Error: ...` plus the context chain."""
    lines = [f"Error: {exc}"]
    seen = {id(exc)}
    cause = exc.__cause__
    while cause is not None and id(cause) not in seen:
        lines.append(f"Caused by: {cause}")
        seen.add(id(cause))
        cause = cause.__cause__
    return "\n".join(lines)
