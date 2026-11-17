from io import StringIO
from pathlib import Path

import pytest
from tests.conftest import git

from wok.cmd.auth import check_auth
from wok.errors import WokError
from wok.git import GitError
from wok.repo import Repo


def test_auth_without_tracking_branch(repo_sample) -> None:
    sample = repo_sample()
    output = StringIO()
    check_auth(sample.repo(), output)
    text = output.getvalue()
    assert "Testing authentication" in text
    assert (
        "No tracking branch" in text
        or "no tracking" in text.lower()
        or "No remote" in text
    )


def test_auth_success_against_local_remote(repo_sample) -> None:
    sample = repo_sample()
    remote = sample.root / "origin.git"
    git(["init", "--bare", remote.name], sample.root)
    git(["remote", "add", "origin", str(remote)], sample.root)
    git(["push", "-u", "origin", "main"], sample.root)
    output = StringIO()
    check_auth(sample.repo(), output)
    assert "Connection successful" in output.getvalue()


def test_auth_failure_reports_troubleshooting(tmp_path: Path) -> None:
    repo = tmp_path / "repo"
    repo.mkdir()
    git(["init", "-b", "main"], repo)
    git(["config", "user.email", "test@localhost"], repo)
    git(["config", "user.name", "Test User"], repo)
    git(["commit", "--allow-empty", "-m", "init"], repo)
    git(["remote", "add", "origin", str(tmp_path / "missing.git")], repo)
    git(["config", "branch.main.remote", "origin"], repo)
    git(["config", "branch.main.merge", "refs/heads/main"], repo)
    output = StringIO()
    with pytest.raises((WokError, GitError)):
        check_auth(Repo.open(repo), output)
    text = output.getvalue()
    assert "failed" in text.lower() or "Failed" in text or "Could not" in text
