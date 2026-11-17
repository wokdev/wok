from tests.conftest import git
from typer.testing import CliRunner

from wok.cli import app

runner = CliRunner()


def test_switch_help_shows_flags() -> None:
    result = runner.invoke(app, ["switch", "--help"])
    assert result.exit_code == 0
    for flag in ("--all", "--create", "-b", "--dirty"):
        assert flag in result.stdout


def test_detached_head_requires_branch(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    git(["checkout", "--detach"], sample.root)
    result = runner.invoke(app, ["-f", str(sample.config_path()), "switch"])
    assert result.exit_code == 1
    assert "detached HEAD" in (result.stderr + result.output)


def test_switch_create_branch_cli(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    git(["add", "."], sample.root)
    git(["commit", "-m", "baseline"], sample.root)
    result = runner.invoke(
        app,
        [
            "-f",
            str(sample.config_path()),
            "switch",
            "-b",
            "feature",
            "--all",
            "--create",
        ],
    )
    assert result.exit_code == 0, result.output + result.stderr
    assert "feature" in (result.stdout + result.output)
    assert (
        git(["branch", "--show-current"], sample.subrepo_path("sub-a")).strip()
        == "feature"
    )


def test_switch_missing_config_fails(repo_sample) -> None:
    sample = repo_sample()
    result = runner.invoke(app, ["-f", str(sample.config_path()), "switch"])
    assert result.exit_code == 1
    assert "not found" in (result.stderr + result.output).lower()
