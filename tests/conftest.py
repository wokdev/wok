"""Temporary wok workspaces for integration tests."""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

import pytest

from wok import DEFAULT_CONFIG_NAME
from wok.repo import Repo

DATA_DIR = Path(__file__).parent / "data"
CONFIGS_DIR = DATA_DIR / "configs"


@pytest.fixture(autouse=True)
def isolated_git_config(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """Keep tests off the developer's git config and allow local file remotes."""
    config = tmp_path / "gitconfig"
    config.write_text(
        "[user]\n"
        "    name = Test User\n"
        "    email = test@localhost\n"
        '[protocol "file"]\n'
        "    allow = always\n"
        "[init]\n"
        "    defaultBranch = main\n"
    )
    monkeypatch.setenv("GIT_CONFIG_GLOBAL", str(config))
    monkeypatch.setenv("GIT_CONFIG_NOSYSTEM", "1")
    monkeypatch.setenv("GIT_CONFIG_SYSTEM", "/dev/null")


class TestRepo:
    """Umbrella repository with optional submodules and a wok.toml copy."""

    def __init__(
        self, root: Path, subrepo_names: list[str], config_name: str | None
    ) -> None:
        self.root = root
        self.subrepo_paths: dict[str, Path] = {}
        init_repo(root)
        for name in subrepo_names:
            self.subrepo_paths[name] = create_submodule(root, name)
        if config_name is not None:
            shutil.copy(CONFIGS_DIR / config_name, root / DEFAULT_CONFIG_NAME)

    def repo(self) -> Repo:
        return Repo.open(self.root)

    def repo_path(self) -> Path:
        return self.root

    def config_path(self) -> Path:
        return self.root / DEFAULT_CONFIG_NAME

    def subrepo_path(self, name: str) -> Path:
        return self.subrepo_paths[name]

    def add_submodule(self, name: str) -> Path:
        path = create_submodule(self.root, name)
        self.subrepo_paths[name] = path
        return path.relative_to(self.root)


@pytest.fixture
def repo_sample(tmp_path: Path):
    def build(
        names: list[str] | None = None, config_name: str | None = None
    ) -> TestRepo:
        return TestRepo(tmp_path / "umbrella", names or [], config_name)

    return build


def expected_config(name: str) -> str:
    return (CONFIGS_DIR / name).read_text()


def git(args: list[str], cwd: Path) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=cwd,
        check=False,
        text=True,
        capture_output=True,
    )
    if result.returncode != 0:
        raise AssertionError(result.stderr or result.stdout or f"git {args} failed")
    return result.stdout


def init_repo(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)
    git(["init", "-b", "main"], path)
    git(["config", "user.email", "test@localhost"], path)
    git(["config", "user.name", "Test User"], path)
    git(["commit", "--allow-empty", "--allow-empty-message", "-m", ""], path)
    git(["branch", "-c", "other"], path)


def create_submodule(repo_path: Path, name: str) -> Path:
    subrepo = repo_path / name
    subrepo.parent.mkdir(parents=True, exist_ok=True)
    init_repo(subrepo)
    git(
        ["-c", "protocol.file.allow=always", "submodule", "add", f"./{name}", name],
        repo_path,
    )
    git(["submodule", "update", "--init", "--", name], repo_path)
    return subrepo


def has_git_lfs() -> bool:
    return shutil.which("git-lfs") is not None


requires_git_lfs = pytest.mark.skipif(
    not has_git_lfs(), reason="git-lfs is not installed"
)
