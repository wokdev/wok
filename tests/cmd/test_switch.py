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


def _switch(sample, create, all_repos, dirty, branch, targets):
    output = StringIO()
    config = Config.load(sample.config_path())
    changed = switch(
        config,
        sample.repo(),
        sample.config_path(),
        output,
        create,
        all_repos,
        dirty,
        branch,
        targets,
    )
    return changed, output.getvalue(), config


def _branch(path: Path) -> str:
    return git(["branch", "--show-current"], path).strip()


def test_switch_all_skips_configured_repo(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b-skip.toml")
    changed, text, _config = _switch(sample, False, True, False, "main", [])
    assert not changed
    assert "Switching 1 repositories for umbrella branch 'main'" in text
    assert "- 'sub-a':" not in text
    assert "- 'sub-b':" in text
    assert "No workspace changes detected; skipping lock" in text
    assert "Successfully processed 1 repositories" in text


def test_switch_all_includes_explicit_repo_overrides_skip(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b-skip.toml")
    changed, text, _config = _switch(
        sample, False, True, False, "main", [Path("sub-a")]
    )
    assert not changed
    assert "Switching 2 repositories for umbrella branch 'main'" in text
    assert "- 'sub-a':" in text
    assert "- 'sub-b':" in text
    assert "No workspace changes detected; skipping lock" in text
    assert "Successfully processed 2 repositories" in text


def test_switch_with_branch_option(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = sample.subrepo_path("sub-a")
    git(["switch", "-c", "develop"], sub)
    git(["switch", "main"], sub)
    changed, text, config = _switch(sample, False, True, False, "develop", [])
    assert changed
    assert config.repos[0].head == "develop"
    assert "Switching 1 repositories for umbrella branch 'develop'" in text
    assert "Locking workspace state" in text
    assert "Successfully switched and locked 1 repositories" in text


def test_switch_updates_file_contents_for_target_branch(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = sample.subrepo_path("sub-a")
    message = sub / "message.txt"
    message.write_text("main\n")
    git(["add", "message.txt"], sub)
    git(["commit", "-m", "Add main content"], sub)
    git(["switch", "-c", "feature"], sub)
    message.write_text("feature\n")
    git(["add", "message.txt"], sub)
    git(["commit", "-m", "Add feature content"], sub)
    git(["switch", "main"], sub)
    assert message.read_text() == "main\n"
    _switch(sample, False, True, False, "feature", [])
    assert message.read_text() == "feature\n"


def test_switch_all_repos_moves_repo_when_cached_head_matches_target(
    repo_sample,
) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = sample.subrepo_path("sub-a")
    git(["switch", "-c", "test"], sample.root)
    git(["switch", "-c", "test"], sub)
    git(["switch", "main"], sub)
    changed, text, config = _switch(sample, False, True, False, "test", [])
    assert changed
    assert config.repos[0].head == "test"
    assert _branch(sub) == "test"
    assert "Locking workspace state" in text
    assert "Successfully switched and locked 1 repositories" in text


def test_switch_with_no_repos(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    changed, text, _config = _switch(sample, False, False, False, "main", [])
    assert not changed
    assert "No repositories to switch" in text


def test_switch_commits_wokfile_with_submodule_state(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    changed, _text, _config = _switch(sample, True, True, False, "feature-branch", [])
    assert changed
    committed = git(["show", "--name-only", "--pretty=format:"], sample.root)
    assert "wok.toml" in committed
    assert "sub-a" in committed
