"""Create wok.toml from existing submodules."""

from __future__ import annotations

import os
from pathlib import Path
from typing import TextIO

from wok.config import Config
from wok.repo import Repo


def init(config_path: Path, umbrella: Repo, stdout: TextIO) -> None:
    wok_config = Config.new()
    for subrepo in umbrella.subrepos:
        repo_path = subrepo.work_dir.relative_to(umbrella.work_dir)
        wok_config.add_repo(repo_path, subrepo.head)
    wok_config.save(config_path)
    stdout.write(f"Created config at `{_display_path(config_path)}`\n")


def _display_path(config_path: Path) -> Path:
    try:
        return config_path.resolve().relative_to(Path(os.getcwd()).resolve())
    except ValueError:
        return config_path
