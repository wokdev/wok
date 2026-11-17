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


def _invoke(sample, *args: str):
    result = runner.invoke(app, ["-f", str(sample.config_path()), "switch", *args])
    detail = result.stdout + result.stderr + result.output
    assert result.exit_code == 0, detail
    return result.stdout, detail


def _branch(path) -> str:
    return git(["branch", "--show-current"], path).strip()


def _commit_wokfile(sample) -> None:
    git(["add", "wok.toml"], sample.root)
    git(["commit", "--allow-empty", "-m", "Add base wokfile"], sample.root)


def test_switch_with_branch_short_flag(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = sample.subrepo_path("sub-a")
    git(["switch", "-c", "develop"], sub)
    git(["switch", "main"], sub)
    _commit_wokfile(sample)
    stdout, _detail = _invoke(sample, "-b", "develop", "--all", "-c")
    assert "Switching 1 repositories for umbrella branch 'develop'" in stdout
    assert "Successfully switched and locked 1 repositories" in stdout


def test_switch_uses_target_branch_wokfile(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b.toml")
    other_config = """version = "1.0"

[[repo]]
path = "sub-a"
head = "other"

[[repo]]
path = "sub-b"
head = "main"
"""
    _commit_wokfile(sample)
    git(["branch", "-f", "other"], sample.root)
    git(["switch", "other"], sample.root)
    sample.config_path().write_text(other_config)
    git(["add", "wok.toml"], sample.root)
    git(["commit", "--allow-empty", "-m", "Update wokfile for other"], sample.root)
    assert 'head = "other"' in sample.config_path().read_text()
    committed = git(["show", "HEAD:wok.toml"], sample.root)
    assert 'head = "other"' in committed
    git(["switch", "main"], sample.root)
    sub_a = sample.subrepo_path("sub-a")
    sub_b = sample.subrepo_path("sub-b")
    git(["switch", "main"], sub_a)
    git(["switch", "main"], sub_b)
    git(["switch", "other"], sub_a)
    git(["switch", "main"], sub_a)
    stdout, _detail = _invoke(sample, "-b", "other")
    assert _branch(sample.root) == "other"
    assert 'head = "other"' in sample.config_path().read_text()
    assert 'head = "other"' in git(["show", "HEAD:wok.toml"], sample.root)
    assert _branch(sub_a) == "other", stdout
    assert _branch(sub_b) == "main"


def test_switch_no_branch_with_explicit_repos(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b.toml")
    sub_a = sample.subrepo_path("sub-a")
    sub_b = sample.subrepo_path("sub-b")
    git(["switch", "-c", "develop"], sub_a)
    git(["switch", "main"], sub_a)
    git(["switch", "-c", "develop"], sub_b)
    git(["switch", "main"], sub_b)
    git(["switch", "-c", "develop"], sample.root)
    _commit_wokfile(sample)
    stdout, _detail = _invoke(sample, "sub-a")
    assert "Switching" in stdout
    assert "develop" in stdout
    assert _branch(sub_a) == "develop"


def test_switch_no_branch_no_repos_reconciles_to_wokfile(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b.toml")
    sub_a = sample.subrepo_path("sub-a")
    sub_b = sample.subrepo_path("sub-b")
    git(["switch", "-c", "develop"], sub_a)
    git(["switch", "-c", "develop"], sub_b)
    git(["switch", "-c", "develop"], sample.root)
    _commit_wokfile(sample)
    _invoke(sample)
    assert _branch(sub_a) == "main"
    assert _branch(sub_b) == "main"


def test_switch_dirty_selects_only_changed_repos(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b.toml")
    sub_a = sample.subrepo_path("sub-a")
    sub_b = sample.subrepo_path("sub-b")
    _commit_wokfile(sample)
    (sub_a / "dirty.txt").write_text("work in progress")
    _invoke(sample, "-cub", "new-feature")
    assert _branch(sub_a) == "new-feature"
    assert (sub_a / "dirty.txt").is_file()
    assert _branch(sub_b) == "main"


def test_switch_dirty_union_with_explicit_repos(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b.toml")
    sub_a = sample.subrepo_path("sub-a")
    sub_b = sample.subrepo_path("sub-b")
    _commit_wokfile(sample)
    (sub_a / "dirty.txt").write_text("work in progress")
    _invoke(sample, "-cub", "new-feature", "sub-b")
    assert _branch(sub_a) == "new-feature"
    assert _branch(sub_b) == "new-feature"
    assert (sub_a / "dirty.txt").is_file()


def test_switch_dirty_no_branch_uses_umbrella_current_branch(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b.toml")
    sub_a = sample.subrepo_path("sub-a")
    sub_b = sample.subrepo_path("sub-b")
    git(["switch", "-c", "feature-branch"], sample.root)
    _commit_wokfile(sample)
    (sub_a / "dirty.txt").write_text("work in progress")
    _invoke(sample, "-cu")
    assert _branch(sub_a) == "feature-branch"
    assert (sub_a / "dirty.txt").is_file()
    assert _branch(sub_b) == "main"
