from io import StringIO
from pathlib import Path

import pytest
from tests.conftest import git

from wok.cmd.assemble import assemble
from wok.config import Config
from wok.errors import WokError


def test_assemble_creates_workspace(tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    (workspace / "component-a").mkdir()
    (workspace / "component-b").mkdir()
    config_path = workspace / "wok.toml"
    assemble(workspace, config_path, StringIO())
    config = Config.load(config_path)
    assert sorted(repo.path.as_posix() for repo in config.repos) == [
        "component-a",
        "component-b",
    ]
    assert all(repo.head == "main" for repo in config.repos)
    modules = (workspace / ".gitmodules").read_text()
    assert "component-a" in modules
    assert "component-b" in modules
    assert (workspace / "component-a" / ".git").is_file()


def test_assemble_is_idempotent(tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    (workspace / "service-a").mkdir()
    (workspace / "service-b").mkdir()
    config_path = workspace / "wok.toml"
    assemble(workspace, config_path, StringIO())
    first = Config.load(config_path).dump()
    assemble(workspace, config_path, StringIO())
    assert Config.load(config_path).dump() == first


def test_assemble_missing_directory(tmp_path: Path) -> None:
    with pytest.raises(WokError, match="does not exist"):
        assemble(tmp_path / "missing", tmp_path / "wok.toml", StringIO())


def test_assemble_preserves_origin(tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"
    component = workspace / "component-a"
    component.mkdir(parents=True)
    git(["init", "-b", "main"], component)
    git(["config", "user.email", "test@localhost"], component)
    git(["config", "user.name", "Test User"], component)
    git(["commit", "--allow-empty", "-m", "init"], component)
    git(
        ["remote", "add", "origin", "https://example.invalid/component-a.git"],
        component,
    )
    assemble(workspace, workspace / "wok.toml", StringIO())
    modules = (workspace / ".gitmodules").read_text()
    assert "https://example.invalid/component-a.git" in modules
