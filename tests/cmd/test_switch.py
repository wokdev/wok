from io import StringIO
from pathlib import Path

import pytest
from tests.conftest import git

from wok.cmd import switch
from wok.config import Config
from wok.errors import WokError


def test_switch_all_repos(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    git(["add", "."], sample.root)
    git(["commit", "-m", "baseline"], sample.root)
    output = StringIO()
    switch(
        Config.load(sample.config_path()),
        sample.repo(),
        sample.config_path(),
        output,
        False,
        True,
        False,
        None,
        [],
    )
    text = output.getvalue()
    assert "Switching 1 repositories" in text
    assert "main" in text
    assert "- 'sub-a':" in text
    assert "Successfully processed 1 repositories" in text


def test_switch_specific_repo(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b.toml")
    output = StringIO()
    config = Config.load(sample.config_path())
    switch(
        config,
        sample.repo(),
        sample.config_path(),
        output,
        False,
        False,
        False,
        None,
        [Path("sub-a")],
    )
    text = output.getvalue()
    assert "- 'sub-a':" in text
    assert "- 'sub-b':" in text


def test_switch_missing_repo_fails(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    with pytest.raises(WokError, match="not in the workspace"):
        switch(
            Config.load(sample.config_path()),
            sample.repo(),
            sample.config_path(),
            StringIO(),
            False,
            False,
            False,
            None,
            [Path("missing")],
        )


def test_switch_creates_branch(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    git(["add", "."], sample.root)
    git(["commit", "-m", "baseline"], sample.root)
    output = StringIO()
    switch(
        Config.load(sample.config_path()),
        sample.repo(),
        sample.config_path(),
        output,
        True,
        True,
        False,
        "feature",
        [],
    )
    text = output.getvalue()
    assert "created and switched" in text or "switched to 'feature'" in text
    assert (
        git(["branch", "--show-current"], sample.subrepo_path("sub-a")).strip()
        == "feature"
    )


def test_switch_refreshes_when_already_on_branch(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = sample.subrepo_path("sub-a")
    (sub / "file.txt").write_text("v1")
    git(["add", "file.txt"], sub)
    git(["commit", "-m", "v1"], sub)
    old = git(["rev-parse", "HEAD"], sub).strip()
    git(["switch", "-c", "hold"], sub)
    (sub / "file.txt").write_text("v2")
    git(["add", "file.txt"], sub)
    git(["commit", "-m", "v2"], sub)
    git(["update-ref", "refs/heads/main", "HEAD"], sub)
    git(["switch", "--discard-changes", "main"], sub)
    assert git(["rev-parse", "HEAD"], sub).strip() != old
    output = StringIO()
    switch(
        Config.load(sample.config_path()),
        sample.repo(),
        sample.config_path(),
        output,
        False,
        False,
        False,
        None,
        [Path("sub-a")],
    )
    assert "already on 'main'" in output.getvalue()
    assert (sub / "file.txt").read_text() == "v2"


def test_switch_all_with_explicit_repos_is_rejected() -> None:
    from typer.testing import CliRunner

    from wok.cli import app

    result = CliRunner().invoke(app, ["switch", "--all", "sub-a"])
    assert result.exit_code == 1
    assert "Cannot specify repositories when using --all" in (
        result.stderr + result.output
    )


def test_switch_dirty_preserves_changes(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = sample.subrepo_path("sub-a")
    (sub / "dirty.txt").write_text("keep")
    output = StringIO()
    switch(
        Config.load(sample.config_path()),
        sample.repo(),
        sample.config_path(),
        output,
        True,
        False,
        True,
        "feature",
        [Path("sub-a")],
    )
    assert (sub / "dirty.txt").read_text() == "keep"
    assert "feature" in output.getvalue()


def test_switch_failure_is_reported(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    output = StringIO()
    switch(
        Config.load(sample.config_path()),
        sample.repo(),
        sample.config_path(),
        output,
        False,
        False,
        False,
        "does-not-exist",
        [Path("sub-a")],
    )
    assert "failed to switch" in output.getvalue()
