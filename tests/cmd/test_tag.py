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


def _tag(
    sample,
    name,
    *,
    sign=False,
    message=None,
    push=False,
    all_repos=False,
    umbrella=True,
    targets=None,
) -> str:
    output = StringIO()
    tag(
        Config.load(sample.config_path()),
        sample.repo(),
        output,
        name,
        sign,
        message,
        push,
        all_repos,
        umbrella,
        list(targets or []),
    )
    return output.getvalue()


def _annotated_message(path: Path, name: str) -> str:
    assert git(["cat-file", "-t", f"refs/tags/{name}"], path).strip() == "tag"
    return git(
        ["for-each-ref", "--format=%(contents)", f"refs/tags/{name}"], path
    ).strip()


def test_tag_create_specific_repo(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    text = _tag(sample, "v1.0.0", targets=[Path("sub-a")])
    assert "Creating tag 'v1.0.0' in 2 repositories" in text
    assert "- 'umbrella': created tag 'v1.0.0'" in text
    assert "- 'sub-a': created tag 'v1.0.0'" in text
    assert "Successfully processed 2 repositories" in text


def test_tag_create_with_sign(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    text = _tag(sample, "v1.0.0", sign=True, all_repos=True)
    assert "Creating tag 'v1.0.0' in 2 repositories" in text
    if "failed to create tag" not in text:
        assert "- 'umbrella': created tag 'v1.0.0'" in text
        assert "- 'sub-a': created tag 'v1.0.0'" in text
        assert "Successfully processed 2 repositories" in text
        return
    assert "failed to create tag 'v1.0.0'" in text
    assert "gpg" in text.lower() or "sign" in text.lower()


def test_tag_create_with_push(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    text = _tag(sample, "v1.0.0", push=True, all_repos=True)
    assert "Creating tag 'v1.0.0' in 2 repositories" in text
    assert "- 'umbrella':" in text
    assert "- 'sub-a': created tag 'v1.0.0'" in text
    assert "Pushing tags to remotes" in text
    assert "Successfully processed 2 repositories" in text


def test_tag_list_all_repos(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    text = _tag(sample, None, all_repos=True)
    assert "Listing tags in" not in text
    assert "✅ umbrella [no-tags]: no tags found" in text
    assert "✅ sub-a [no-tags]: no tags found" in text
    assert "Successfully processed" not in text


def test_tag_list_specific_repo(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    text = _tag(sample, None, targets=[Path("sub-a")])
    assert "Listing tags in" not in text
    assert "✅ umbrella [no-tags]: no tags found" in text
    assert "✅ sub-a [no-tags]: no tags found" in text
    assert "Successfully processed" not in text


def test_tag_list_skips_umbrella_when_disabled(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    text = _tag(sample, None, all_repos=True, umbrella=False)
    assert "Listing tags in" not in text
    assert " umbrella:" not in text
    assert "✅ sub-a [no-tags]: no tags found" in text
    assert "Successfully processed" not in text


def test_tag_default_skips_configured_repo(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b-skip.toml")
    text = _tag(sample, None)
    assert "Listing tags in" not in text
    assert "✅ umbrella [no-tags]: no tags found" in text
    assert " sub-a:" not in text
    assert "✅ sub-b [no-tags]: no tags found" in text
    assert "Successfully processed" not in text


def test_tag_multiple_repos(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b.toml")
    text = _tag(sample, "v1.0.0", all_repos=True)
    assert "Creating tag 'v1.0.0' in 3 repositories" in text
    assert "- 'umbrella':" in text
    assert "- 'sub-a': created tag 'v1.0.0'" in text
    assert "- 'sub-b': created tag 'v1.0.0'" in text
    assert "Successfully processed 3 repositories" in text


def test_tag_specific_multiple_repos(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b.toml")
    text = _tag(sample, "v1.0.0", targets=[Path("sub-a"), Path("sub-b")])
    assert "Creating tag 'v1.0.0' in 3 repositories" in text
    assert "- 'umbrella': created tag 'v1.0.0'" in text
    assert "- 'sub-a': created tag 'v1.0.0'" in text
    assert "- 'sub-b': created tag 'v1.0.0'" in text
    assert "Successfully processed 3 repositories" in text


def test_tag_with_no_repos(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    text = _tag(sample, None)
    assert "Listing tags in" not in text
    assert "✅ umbrella [no-tags]: no tags found" in text
    assert "Successfully processed" not in text


def test_tag_no_repos_without_umbrella(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    text = _tag(sample, None, all_repos=False, umbrella=False)
    assert "No repositories to tag" in text
    assert "- 'umbrella':" not in text


def test_tag_all_with_repos_is_rejected() -> None:
    from typer.testing import CliRunner

    from wok.cli import app

    result = CliRunner().invoke(app, ["tag", "v1.0.0", "--all", "sub-a"])
    assert result.exit_code == 1
    assert "Cannot specify repositories when using --all" in (
        result.stderr + result.output
    )


def _invoke(sample, *args: str):
    from typer.testing import CliRunner

    from wok.cli import app

    result = CliRunner().invoke(app, ["-f", str(sample.config_path()), "tag", *args])
    return result


def test_tag_implicit_create_with_sign_flag(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    result = _invoke(sample, "v1.0.0", "--sign", "--all")
    combined = result.stdout + result.stderr + result.output
    assert "Creating tag 'v1.0.0'" in result.stdout or "GPG" in combined


def test_tag_implicit_create_with_message_flag(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    result = _invoke(sample, "v1.0.0", "--message", "Release v1.0.0", "--all")
    assert result.exit_code == 0, result.output + result.stderr
    assert "Creating tag 'v1.0.0'" in result.stdout
    assert "created tag 'v1.0.0'" in result.stdout
    assert _annotated_message(sample.root, "v1.0.0") == "Release v1.0.0"


def test_tag_implicit_create_with_message_short_flag(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    result = _invoke(sample, "v2.0.0", "-m", "Release v2.0.0", "--all")
    assert result.exit_code == 0, result.output + result.stderr
    assert "Creating tag 'v2.0.0'" in result.stdout
    assert "created tag 'v2.0.0'" in result.stdout
    assert _annotated_message(sample.root, "v2.0.0") == "Release v2.0.0"


def test_tag_explicit_create_with_message_flag(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    result = _invoke(sample, "--all", "create", "v3.0.0", "-m", "Release v3.0.0")
    assert result.exit_code == 0, result.output + result.stderr
    assert "Creating tag 'v3.0.0'" in result.stdout
    assert "created tag 'v3.0.0'" in result.stdout
    assert _annotated_message(sample.root, "v3.0.0") == "Release v3.0.0"


def test_tag_list_subcommand(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    created = _invoke(sample, "--all", "create", "v1.0.0")
    assert created.exit_code == 0, created.output + created.stderr
    result = _invoke(sample, "--all", "list")
    assert result.exit_code == 0, result.output + result.stderr
    assert "✅ umbrella [v1.0.0]: v1.0.0" in result.stdout
    assert "✅ sub-a [v1.0.0]: v1.0.0" in result.stdout


def test_tag_implicit_list(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    created = _invoke(sample, "--all", "create", "v1.0.0")
    assert created.exit_code == 0, created.output + created.stderr
    result = _invoke(sample, "--all")
    assert result.exit_code == 0, result.output + result.stderr
    assert "✅ umbrella [v1.0.0]: v1.0.0" in result.stdout
    assert "✅ sub-a [v1.0.0]: v1.0.0" in result.stdout
    listed = _invoke(sample, "sub-a")
    assert listed.exit_code == 0, listed.output + listed.stderr
    assert "✅ sub-a [v1.0.0]: v1.0.0" in listed.stdout


def test_tag_create_help_shows_options() -> None:
    from typer.testing import CliRunner

    from wok.cli import app

    result = CliRunner().invoke(app, ["tag", "create", "--help"])
    assert result.exit_code == 0
    assert "--sign" in result.stdout or "-s" in result.stdout
    assert "--message" in result.stdout or "-m" in result.stdout


def test_tag_help_mentions_subcommands() -> None:
    from typer.testing import CliRunner

    from wok.cli import app

    result = CliRunner().invoke(app, ["tag", "--help"])
    assert result.exit_code == 0
    assert "create" in result.stdout
    assert "list" in result.stdout
    assert "push" in result.stdout
    assert "-s" not in result.stdout or "Sign the tag" not in result.stdout
