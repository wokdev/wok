from io import StringIO

from tests.conftest import git

from wok.cmd import lock
from wok.config import Config
from wok.errors import WokError


def _count(path) -> int:
    return int(git(["rev-list", "--count", "HEAD"], path).strip())


def test_lock_with_no_changes(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    git(["add", "."], sample.root)
    git(["commit", "-m", "Initial setup"], sample.root)
    before = _count(sample.root)
    output = StringIO()
    lock(Config.load(sample.config_path()), sample.repo(), output)
    assert output.getvalue() == "No submodule changes detected; nothing to lock\n"
    assert _count(sample.root) == before


def test_lock_with_no_submodules(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    before = _count(sample.root)
    output = StringIO()
    lock(Config.load(sample.config_path()), sample.repo(), output)
    assert output.getvalue() == "No submodule changes detected; nothing to lock\n"
    assert _count(sample.root) == before


def test_lock_truncates_long_subject(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    git(["add", "."], sample.root)
    git(["commit", "-m", "Initial setup"], sample.root)
    before = _count(sample.root)
    message = (
        "Test commit message for submodule that is quite long "
        "and should be truncated properly"
    )
    git(["commit", "--allow-empty", "-m", message], sample.subrepo_path("sub-a"))
    output = StringIO()
    repo = sample.repo()
    lock(Config.load(sample.config_path()), repo, output)
    assert output.getvalue() == "Locked submodule state\n"
    assert _count(sample.root) == before + 1
    committed = git(["log", "-1", "--format=%B"], sample.root)
    assert "Lock submodule state" in committed
    assert "Changed submodules:" in committed
    assert "sub-a: Test commit message for submodule that is quite..." in committed


def test_lock_keeps_short_subject(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    git(["add", "."], sample.root)
    git(["commit", "-m", "Initial setup"], sample.root)
    git(
        ["commit", "--allow-empty", "-m", "Short message"], sample.subrepo_path("sub-a")
    )
    lock(Config.load(sample.config_path()), sample.repo(), StringIO())
    committed = git(["log", "-1", "--format=%B"], sample.root)
    assert "sub-a: Short message" in committed
    assert "..." not in committed


def test_lock_runs_pre_commit_hook(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    git(["add", "."], sample.root)
    git(["commit", "-m", "Initial setup"], sample.root)
    git(["commit", "--allow-empty", "-m", "hook me"], sample.subrepo_path("sub-a"))
    hooks = sample.root / ".git" / "hooks"
    hooks.mkdir(exist_ok=True)
    marker = sample.root / "hook-ran"
    (hooks / "pre-commit").write_text(f"#!/bin/sh\ntouch {marker}\n")
    (hooks / "pre-commit").chmod(0o755)
    lock(Config.load(sample.config_path()), sample.repo(), StringIO())
    assert marker.exists()


def test_failing_pre_commit_hook_aborts_lock(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    git(["add", "."], sample.root)
    git(["commit", "-m", "Initial setup"], sample.root)
    before = git(["rev-parse", "HEAD"], sample.root).strip()
    git(["commit", "--allow-empty", "-m", "hook me"], sample.subrepo_path("sub-a"))
    hooks = sample.root / ".git" / "hooks"
    hooks.mkdir(exist_ok=True)
    (hooks / "pre-commit").write_text("#!/bin/sh\necho hook-failed >&2\nexit 1\n")
    (hooks / "pre-commit").chmod(0o755)
    import pytest

    with pytest.raises(WokError, match="lock"):
        lock(Config.load(sample.config_path()), sample.repo(), StringIO())
    assert git(["rev-parse", "HEAD"], sample.root).strip() == before
