from io import StringIO
from pathlib import Path

from tests.conftest import git

from wok.cmd import status
from wok.config import Config


def test_status_clean_without_submodules(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, False)
    assert output.getvalue() == "✅ umbrella [main]: clean\n"


def test_status_clean_with_submodule(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, False)
    assert output.getvalue() == "✅ umbrella [main]: clean\n✅ sub-a [main]: clean\n"


def _publish(sample, name: str = "sub-a") -> Path:
    sub = sample.subrepo_path(name)
    git(["add", "."], sample.root)
    git(["commit", "-m", "baseline"], sample.root)
    remote_parent = sample.root / "remotes"
    remote_parent.mkdir()
    remote = remote_parent / f"{name}.git"
    git(["init", "--bare", remote.name], remote_parent)
    git(["remote", "add", "origin", str(remote)], sub)
    git(["push", "-u", "origin", "main"], sub)
    return sub


def test_status_up_to_date_with_remote(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    _publish(sample)
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, False)
    assert "✅ sub-a [main]: clean" in output.getvalue()


def test_status_ahead_of_remote_is_new_commits(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = _publish(sample)
    (sub / "LOCAL.md").write_text("local")
    git(["add", "LOCAL.md"], sub)
    git(["commit", "-m", "local"], sub)
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, False)
    assert "⬆ sub-a [main]: new commits" in output.getvalue()


def test_status_multiple_local_commits(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = _publish(sample)
    for index in (1, 2):
        (sub / f"LOCAL{index}.md").write_text(str(index))
        git(["add", f"LOCAL{index}.md"], sub)
        git(["commit", "-m", f"local {index}"], sub)
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, False)
    assert "⬆ sub-a [main]: new commits" in output.getvalue()


def test_status_behind_remote_stays_clean(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = _publish(sample)
    remote = sample.root / "remotes" / "sub-a.git"
    clone = sample.root / "remotes" / "contributor"
    git(["clone", str(remote), str(clone)], sample.root / "remotes")
    git(["config", "user.email", "test@localhost"], clone)
    git(["config", "user.name", "Test User"], clone)
    (clone / "UPSTREAM.md").write_text("upstream")
    git(["add", "UPSTREAM.md"], clone)
    git(["commit", "-m", "upstream"], clone)
    git(["push"], clone)
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, True)
    text = output.getvalue()
    assert "✅ sub-a [main]: clean" in text
    assert sub.exists()


def test_status_diverged_still_reports_new_commits(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = _publish(sample)
    (sub / "LOCAL.md").write_text("local")
    git(["add", "LOCAL.md"], sub)
    git(["commit", "-m", "local"], sub)
    clone = sample.root / "remotes" / "contributor"
    git(
        ["clone", str(sample.root / "remotes" / "sub-a.git"), str(clone)],
        sample.root / "remotes",
    )
    git(["config", "user.email", "test@localhost"], clone)
    git(["config", "user.name", "Test User"], clone)
    (clone / "UPSTREAM.md").write_text("upstream")
    git(["add", "UPSTREAM.md"], clone)
    git(["commit", "-m", "upstream"], clone)
    git(["push"], clone)
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, True)
    assert "⬆ sub-a [main]: new commits" in output.getvalue()


def test_status_without_tracking_branch(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, False)
    assert "✅ sub-a [main]: clean" in output.getvalue()
    assert "no remote" not in output.getvalue()


def test_status_fetch_updates_remote_refs(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    _publish(sample)
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, True)
    assert "✅ sub-a [main]: clean" in output.getvalue()


def test_status_umbrella_ahead_of_remote(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    git(["add", "."], sample.root)
    git(["commit", "-m", "baseline"], sample.root)
    remote = sample.root.parent / "origin.git"
    git(["init", "--bare", str(remote)], sample.root.parent)
    git(["remote", "add", "origin", str(remote)], sample.root)
    git(["push", "-u", "origin", "main"], sample.root)
    git(["commit", "--allow-empty", "-m", "ahead"], sample.root)
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, False)
    assert "⬆ umbrella [main]: new commits" in output.getvalue()


def test_status_umbrella_only_behind_stays_clean(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    git(["add", "."], sample.root)
    git(["commit", "-m", "baseline"], sample.root)
    remote = sample.root.parent / "origin.git"
    git(["init", "--bare", str(remote)], sample.root.parent)
    git(["remote", "add", "origin", str(remote)], sample.root)
    git(["push", "-u", "origin", "main"], sample.root)
    clone = sample.root.parent / "clone"
    git(["clone", str(remote), str(clone)], sample.root.parent)
    git(["config", "user.email", "test@localhost"], clone)
    git(["config", "user.name", "Test User"], clone)
    git(["commit", "--allow-empty", "-m", "upstream"], clone)
    git(["push"], clone)
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, True)
    assert "✅ umbrella [main]: clean" in output.getvalue()


def test_status_other_branch_without_changes(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    git(["switch", "-c", "other-branch"], sample.subrepo_path("sub-a"))
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, False)
    text = output.getvalue()
    assert "sub-a [other-branch]" in text
    assert "clean" in text


def test_status_dirty_worktree(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    (sample.subrepo_path("sub-a") / "dirty.txt").write_text("x")
    output = StringIO()
    status(Config.load(sample.config_path()), sample.repo(), output, False)
    assert "❌ sub-a [main]: has uncommitted changes" in output.getvalue()


def test_status_missing_wokfile(repo_sample) -> None:
    from typer.testing import CliRunner

    from wok.cli import app

    sample = repo_sample()
    result = CliRunner().invoke(app, ["-f", str(sample.config_path()), "status"])
    assert result.exit_code == 1
    assert "not found" in (result.stderr + result.output).lower() or "Cannot read" in (
        result.stderr + result.output
    )
