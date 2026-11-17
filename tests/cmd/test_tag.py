from io import StringIO
from pathlib import Path

import pytest
from tests.conftest import git

from wok.cmd import tag, tag_create, tag_list, tag_push
from wok.config import Config
from wok.errors import WokError


def test_tag_list_empty(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    output = StringIO()
    tag_list(Config.load(sample.config_path()), sample.repo(), output, False, True, [])
    assert "✅ umbrella [no-tags]: no tags found" in output.getvalue()


def test_tag_create_all(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    output = StringIO()
    tag_create(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        "v1.0.0",
        False,
        None,
        False,
        True,
        True,
        [],
    )
    text = output.getvalue()
    assert "Creating tag 'v1.0.0' in 2 repositories" in text
    assert "- 'umbrella': created tag 'v1.0.0'" in text
    assert "- 'sub-a': created tag 'v1.0.0'" in text
    assert "Successfully processed 2 repositories" in text
    assert "refs/tags/v1.0.0" in git(["show-ref", "--tags"], sample.root)


def test_tag_create_already_exists(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    git(["tag", "v1.0.0"], sample.root)
    output = StringIO()
    tag_create(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        "v1.0.0",
        False,
        None,
        False,
        True,
        False,
        [],
    )
    assert "already exists" in output.getvalue()


def test_tag_create_with_message(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    output = StringIO()
    tag(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        "v1.0.0",
        False,
        "Release version 1.0.0",
        False,
        True,
        True,
        [],
    )
    message = git(
        ["for-each-ref", "--format=%(contents)", "refs/tags/v1.0.0"], sample.root
    )
    assert "Release version 1.0.0" in message


def test_tag_list_sorts_newest_first(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    git(["tag", "z-older"], sample.root)
    (sample.root / "AFTER_TAG.md").write_text("after")
    git(["add", "AFTER_TAG.md"], sample.root)
    git(["commit", "-m", "commit after first tag"], sample.root)
    git(["tag", "a-newer"], sample.root)
    output = StringIO()
    tag(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        None,
        False,
        None,
        False,
        False,
        True,
        [],
    )
    assert "✅ umbrella [a-newer]: a-newer, z-older" in output.getvalue()


def test_tag_list_ahead_marker(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    git(["tag", "v1.0.0"], sample.root)
    (sample.root / "AFTER.md").write_text("after")
    git(["add", "AFTER.md"], sample.root)
    git(["commit", "-m", "commit after latest tag"], sample.root)
    output = StringIO()
    tag(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        None,
        False,
        None,
        False,
        False,
        True,
        [],
    )
    assert "⬆ umbrella [v1.0.0-1]: v1.0.0" in output.getvalue()


def test_tag_skips_configured_repo(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b-skip.toml")
    output = StringIO()
    tag_list(Config.load(sample.config_path()), sample.repo(), output, True, True, [])
    text = output.getvalue()
    assert "sub-a" not in text
    assert "sub-b" in text


def test_tag_explicit_overrides_skip(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b-skip.toml")
    output = StringIO()
    tag(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        "v1.0.0",
        False,
        None,
        False,
        True,
        True,
        [Path("sub-a")],
    )
    assert "- 'sub-a': created tag 'v1.0.0'" in output.getvalue()


def test_tag_missing_repo_fails(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    with pytest.raises(WokError, match="not in the workspace"):
        tag_create(
            Config.load(sample.config_path()),
            sample.repo(),
            StringIO(),
            "v1.0.0",
            False,
            None,
            False,
            False,
            False,
            [Path("missing")],
        )


def test_tag_push_no_remote(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    git(["tag", "v1.0.0"], sample.root)
    output = StringIO()
    tag_push(Config.load(sample.config_path()), sample.repo(), output, False, True, [])
    assert "No remote" in output.getvalue() or "failed" in output.getvalue()


def test_tag_push_to_bare_remote(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    remote = sample.root / "origin.git"
    git(["init", "--bare", remote.name], sample.root)
    git(["remote", "add", "origin", str(remote)], sample.root)
    git(["push", "-u", "origin", "main"], sample.root)
    git(["tag", "v1.0.0"], sample.root)
    output = StringIO()
    tag_push(Config.load(sample.config_path()), sample.repo(), output, False, True, [])
    text = output.getvalue()
    assert "pushed tags" in text
    assert "refs/tags/v1.0.0" in git(["ls-remote", "--tags", "origin"], sample.root)


def test_tag_push_nothing_to_push(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    remote = sample.root / "origin.git"
    git(["init", "--bare", remote.name], sample.root)
    git(["remote", "add", "origin", str(remote)], sample.root)
    output = StringIO()
    tag_push(Config.load(sample.config_path()), sample.repo(), output, False, True, [])
    assert "no tags to push" in output.getvalue()


def test_implicit_create_via_cli(repo_sample) -> None:
    from typer.testing import CliRunner

    from wok.cli import app

    sample = repo_sample(["sub-a"], "a.toml")
    result = CliRunner().invoke(
        app,
        [
            "-f",
            str(sample.config_path()),
            "tag",
            "v1.0.0",
            "--message",
            "Release v1.0.0",
            "--all",
        ],
    )
    assert result.exit_code == 0, result.output + result.stderr
    assert "created tag 'v1.0.0'" in result.stdout
    message = git(
        ["for-each-ref", "--format=%(contents)", "refs/tags/v1.0.0"], sample.root
    )
    assert "Release v1.0.0" in message


def test_tag_all_with_repos_is_rejected() -> None:
    from typer.testing import CliRunner

    from wok.cli import app

    result = CliRunner().invoke(app, ["tag", "v1.0.0", "--all", "sub-a"])
    assert result.exit_code == 1
    assert "Cannot specify repositories when using --all" in (
        result.stderr + result.output
    )
