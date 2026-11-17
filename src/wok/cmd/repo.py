"""Add and remove repositories in wok.toml."""

from __future__ import annotations

from pathlib import Path

from wok.config import Config
from wok.errors import WokError
from wok.repo import Repo


def add(wok_config: Config, umbrella: Repo, submodule_path: Path) -> bool:
    subrepo_path = umbrella.work_dir / submodule_path
    subrepo = umbrella.get_subrepo_by_path(
        submodule_path
    ) or umbrella.get_subrepo_by_path(subrepo_path)
    if subrepo is None:
        raise WokError(f"Cannot find submodule at `{subrepo_path}`")
    relative = subrepo.work_dir.relative_to(umbrella.work_dir)
    if not wok_config.add_repo(relative, subrepo.head):
        print(f"Not adding existing subrepo at `{subrepo.work_dir}`")
        return False
    print(f"Added subrepo at `{subrepo.work_dir}` with head `{subrepo.head}`")
    return True


def rm(wok_config: Config, submodule_path: Path) -> bool:
    if not wok_config.remove_repo(submodule_path):
        print(f"No subrepo at `{submodule_path}` in config")
        return False
    print(f"Removed subrepo at `{submodule_path}`")
    return True
