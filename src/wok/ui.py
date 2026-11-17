"""Live per-repo sections and ordered parallel execution."""

from __future__ import annotations

import asyncio
import contextlib
import sys
from collections.abc import Awaitable, Callable, Sequence
from contextvars import ContextVar
from typing import TypeVar

from rich.console import Console, Group
from rich.live import Live
from rich.spinner import Spinner
from rich.text import Text

from wok.runtime import disable_prompt, inherit_tty

T = TypeVar("T")
R = TypeVar("R")

section_var: ContextVar[Section | None] = ContextVar("wok_section", default=None)

_VISIBLE_LINES = 8


class Section:
    """Rolling log for one repository."""

    def __init__(self, name: str) -> None:
        self.name = name
        self.lines: list[str] = []
        self._current = ""
        self.done = False
        self.ok = True
        self.error: BaseException | None = None

    def feed(self, text: str) -> None:
        """Treat each carriage return as an update to the current line."""
        self._current += text
        while "\n" in self._current or "\r" in self._current:
            newline = self._current.find("\n")
            carriage = self._current.find("\r")
            if carriage != -1 and (newline == -1 or carriage < newline):
                self._current = self._current[carriage + 1 :]
                continue
            line, self._current = self._current.split("\n", 1)
            self.lines.append(line.split("\r")[-1])

    def display_lines(self, limit: int = _VISIBLE_LINES) -> list[str]:
        visible = list(self.lines)
        if self._current:
            visible.append(self._current.split("\r")[-1])
        return visible[-limit:]

    def full_log(self) -> str:
        visible = list(self.lines)
        if self._current:
            visible.append(self._current.split("\r")[-1])
        return "\n".join(visible)


def _render(sections: Sequence[Section]) -> Group:
    renderables: list[Spinner | Text] = []
    for section in sections:
        if section.done and section.ok:
            renderables.append(Text(f"✓ {section.name}"))
            continue
        if section.done:
            renderables.append(Text(f"✗ {section.name}", style="red"))
        else:
            renderables.append(Spinner("dots", text=section.name))
        renderables.extend(Text(f"  {line}") for line in section.display_lines())
    if not renderables:
        renderables.append(Text(""))
    return Group(*renderables)


def _print_block(section: Section) -> None:
    status = "ok" if section.ok else "failed"
    print(f"== {section.name} ({status}) ==", file=sys.stderr)
    log = section.full_log()
    if log:
        print(log, file=sys.stderr)


async def run_parallel(
    items: Sequence[T],
    fn: Callable[[T], Awaitable[R]],
    jobs: int,
    *,
    label: Callable[[T], str] = str,
) -> list[R | BaseException]:
    """Run `fn` over items with a semaphore.

    Results stay in input order. A terminal shows Rich sections; otherwise each
    repo prints one plain block as it finishes. `-j1` inherits the terminal.
    """
    jobs = max(1, jobs)
    semaphore = asyncio.Semaphore(jobs)
    inherit = jobs == 1
    prompt_off = jobs > 1
    use_live = prompt_off and sys.stderr.isatty()
    sections = [Section(label(item)) for item in items]

    async def one(item: T, section: Section) -> R:
        async with semaphore:
            token_section = section_var.set(None if inherit else section)
            token_inherit = inherit_tty.set(inherit)
            token_prompt = disable_prompt.set(prompt_off)
            try:
                return await fn(item)
            except Exception as exc:
                section.ok = False
                section.error = exc
                raise
            finally:
                section.done = True
                section_var.reset(token_section)
                inherit_tty.reset(token_inherit)
                disable_prompt.reset(token_prompt)
                if not use_live and not inherit:
                    _print_block(section)

    async def gather() -> list[R | BaseException]:
        return await asyncio.gather(
            *(
                one(item, section)
                for item, section in zip(items, sections, strict=True)
            ),
            return_exceptions=True,
        )

    if not use_live:
        return await gather()

    console = Console(file=sys.stderr, force_terminal=True)
    with Live(_render(sections), console=console, refresh_per_second=8) as live:
        stop = asyncio.Event()

        async def refresher() -> None:
            while not stop.is_set():
                live.update(_render(sections))
                try:
                    await asyncio.wait_for(stop.wait(), timeout=0.1)
                except TimeoutError:
                    continue

        task = asyncio.create_task(refresher())
        results = await gather()
        stop.set()
        task.cancel()
        with contextlib.suppress(asyncio.CancelledError):
            await task
        live.update(_render(sections))

    for section in sections:
        if not section.ok:
            log = section.full_log()
            if log:
                print(log, file=sys.stderr)
    return results
