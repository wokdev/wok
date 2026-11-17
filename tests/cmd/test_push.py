from io import StringIO
from pathlib import Path

import pytest
from tests.conftest import git

from wok.cmd import push
from wok.config import Config
from wok.errors import WokError


def test_push_all_without_remote(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    output = StringIO()
    push(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        False,
        True,
        None,
        True,
        [],
    )
    text = output.getvalue()
    assert "Pushing 2 repositories to branch 'main'" in text
    assert "- 'umbrella': no remote configured, skipping" in text
    assert "- 'sub-a': no remote configured, skipping" in text
    assert "Successfully processed 2 repositories" in text


def test_push_specific_repo(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b.toml")
    output = StringIO()
    push(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        False,
        False,
        None,
        False,
        [Path("sub-a")],
    )
    text = output.getvalue()
    assert "- 'sub-a':" in text
    assert "sub-b" not in text


def test_push_missing_repo_fails(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    with pytest.raises(WokError, match="not in the workspace"):
        push(
            Config.load(sample.config_path()),
            sample.repo(),
            StringIO(),
            False,
            False,
            None,
            False,
            [Path("missing")],
        )


def test_push_to_bare_remote(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    git(["add", "."], sample.root)
    git(["commit", "-m", "baseline"], sample.root)
    remote = sample.root / "remotes" / "sub-a.git"
    remote.parent.mkdir()
    git(["init", "--bare", remote.name], remote.parent)
    git(["remote", "add", "origin", str(remote)], sample.subrepo_path("sub-a"))
    output = StringIO()
    push(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        True,
        False,
        None,
        False,
        [Path("sub-a")],
    )
    text = output.getvalue()
    assert "pushed to 'main'" in text
    assert (
        git(["rev-parse", "refs/heads/main"], remote).strip()
        == git(["rev-parse", "HEAD"], sample.subrepo_path("sub-a")).strip()
    )


def test_push_already_up_to_date(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    remote = sample.root / "sub-a.git"
    git(["init", "--bare", remote.name], sample.root)
    git(["remote", "add", "origin", str(remote)], sample.subrepo_path("sub-a"))
    git(["push", "-u", "origin", "main"], sample.subrepo_path("sub-a"))
    output = StringIO()
    push(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        False,
        False,
        None,
        False,
        [Path("sub-a")],
    )
    assert "already up to date" in output.getvalue()


def test_push_skips_configured_repo(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b-skip.toml")
    output = StringIO()
    push(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        False,
        True,
        None,
        False,
        [],
    )
    text = output.getvalue()
    assert "sub-a" not in text
    assert "sub-b" in text


def test_push_explicit_overrides_skip(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a-b-skip.toml")
    output = StringIO()
    push(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        False,
        True,
        None,
        False,
        [Path("sub-a")],
    )
    assert "sub-a" in output.getvalue()


def test_push_all_with_repos_is_rejected() -> None:
    from typer.testing import CliRunner

    from wok.cli import app

    result = CliRunner().invoke(app, ["push", "--all", "sub-a"])
    assert result.exit_code == 1
    assert "Cannot specify repositories when using --all" in (
        result.stderr + result.output
    )


def test_push_no_repositories(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    output = StringIO()
    push(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        False,
        False,
        None,
        False,
        [],
    )
    assert output.getvalue() == "No repositories to push\n"


def test_pre_push_hook_runs(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    remote = sample.root / "sub-a.git"
    git(["init", "--bare", remote.name], sample.root)
    sub = sample.subrepo_path("sub-a")
    git(["remote", "add", "origin", str(remote)], sub)
    hooks = sub / ".git" / "hooks"
    hooks.mkdir(exist_ok=True)
    marker = sample.root / "pushed"
    (hooks / "pre-push").write_text(f"#!/bin/sh\ntouch {marker}\n")
    (hooks / "pre-push").chmod(0o755)
    output = StringIO()
    push(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        False,
        False,
        None,
        False,
        [Path("sub-a")],
    )
    assert marker.exists()
    assert "pushed to 'main'" in output.getvalue()


def test_failing_pre_push_hook_is_reported(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    remote = sample.root / "sub-a.git"
    git(["init", "--bare", remote.name], sample.root)
    sub = sample.subrepo_path("sub-a")
    git(["remote", "add", "origin", str(remote)], sub)
    hooks = sub / ".git" / "hooks"
    hooks.mkdir(exist_ok=True)
    (hooks / "pre-push").write_text("#!/bin/sh\necho blocked >&2\nexit 1\n")
    (hooks / "pre-push").chmod(0o755)
    output = StringIO()
    push(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        False,
        False,
        None,
        False,
        [Path("sub-a")],
    )
    assert "failed" in output.getvalue()
    assert not (remote / "refs" / "heads" / "main").exists()


def _push(
    sample, *, upstream=False, all_repos=False, branch=None, umbrella=True, targets=None
):
    output = StringIO()
    push(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        upstream,
        all_repos,
        branch,
        umbrella,
        list(targets or []),
    )
    return output.getvalue()


def test_push_all_repos(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    text = _push(sample, all_repos=True)
    assert "Pushing 2 repositories to branch 'main'" in text
    assert "- 'umbrella':" in text
    assert "- 'sub-a':" in text
    assert (
        "pushed to 'main'" in text
        or "already up to date" in text
        or "no remote configured, skipping" in text
    )
    assert "Successfully processed 2 repositories" in text


def test_push_multiple_repos(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b.toml")
    text = _push(sample, all_repos=True)
    assert "Pushing 3 repositories to branch 'main'" in text
    assert "- 'umbrella':" in text
    assert "- 'sub-a':" in text
    assert "- 'sub-b':" in text
    assert "Successfully processed 3 repositories" in text


def test_push_specific_multiple_repos(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b.toml")
    text = _push(sample, targets=[Path("sub-a"), Path("sub-b")])
    assert "Pushing 3 repositories to branch 'main'" in text
    assert "- 'umbrella':" in text
    assert "- 'sub-a':" in text
    assert "- 'sub-b':" in text
    assert "Successfully processed 3 repositories" in text


def test_push_skips_umbrella_when_disabled(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    text = _push(sample, all_repos=True, umbrella=False)
    assert "Pushing 1 repositories to branch 'main'" in text
    assert "- 'umbrella':" not in text
    assert "- 'sub-a':" in text
    assert "Successfully processed 1 repositories" in text


def test_push_with_branch_option(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    text = _push(sample, branch="develop")
    assert "Pushing 2 repositories to branch 'develop'" in text
    assert "- 'umbrella':" in text
    assert "Successfully processed 2 repositories" in text


def test_push_with_upstream_option(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    text = _push(sample, upstream=True)
    assert "Pushing 2 repositories to branch 'main'" in text
    assert "- 'umbrella':" in text
    assert (
        "pushed to 'main' and set upstream" in text
        or "already up to date" in text
        or "no remote configured, skipping" in text
    )
    assert "Successfully processed 2 repositories" in text


def test_push_with_no_repos(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    text = _push(sample)
    assert "Pushing 1 repositories to branch 'main'" in text
    assert "- 'umbrella':" in text
    assert "Successfully processed 1 repositories" in text
