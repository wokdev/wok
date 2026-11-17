from io import StringIO
from pathlib import Path

from tests.conftest import git

from wok.cmd import update
from wok.config import Config


def test_update_without_remote(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    output = StringIO()
    update(Config.load(sample.config_path()), sample.repo(), output, False, True)
    text = output.getvalue()
    assert "Updating repositories" in text
    assert "already up to date" in text


def test_update_fast_forwards(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    git(["add", "."], sample.root)
    git(["commit", "-m", "baseline"], sample.root)
    sub = sample.subrepo_path("sub-a")
    remote = sample.root / "remotes" / "sub-a.git"
    remote.parent.mkdir()
    git(["init", "--bare", remote.name], remote.parent)
    git(["remote", "add", "origin", str(remote)], sub)
    git(["push", "-u", "origin", "main"], sub)
    clone = sample.root / "remotes" / "contributor"
    git(["clone", str(remote), str(clone)], remote.parent)
    git(["config", "user.email", "test@localhost"], clone)
    git(["config", "user.name", "Test User"], clone)
    (clone / "UPSTREAM.md").write_text("upstream")
    git(["add", "UPSTREAM.md"], clone)
    git(["commit", "-m", "upstream"], clone)
    git(["push"], clone)
    output = StringIO()
    update(Config.load(sample.config_path()), sample.repo(), output, False, True)
    text = output.getvalue()
    assert "fast-forwarded 'main'" in text
    assert "Updated submodule state committed" in text
    assert "UPSTREAM.md" in git(["ls-tree", "-r", "--name-only", "HEAD"], sub)


def test_update_respects_rebase(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = sample.subrepo_path("sub-a")
    git(["config", "pull.rebase", "true"], sub)
    git(["add", "."], sample.root)
    git(["commit", "-m", "baseline"], sample.root)
    remote = sample.root / "remotes" / "sub-a.git"
    remote.parent.mkdir()
    git(["init", "--bare", remote.name], remote.parent)
    git(["remote", "add", "origin", str(remote)], sub)
    git(["push", "-u", "origin", "main"], sub)
    (sub / "LOCAL.md").write_text("local")
    git(["add", "LOCAL.md"], sub)
    git(["commit", "-m", "local commit"], sub)
    clone = sample.root / "remotes" / "contributor"
    git(["clone", str(remote), str(clone)], remote.parent)
    git(["config", "user.email", "test@localhost"], clone)
    git(["config", "user.name", "Test User"], clone)
    (clone / "UPSTREAM.md").write_text("upstream")
    git(["add", "UPSTREAM.md"], clone)
    git(["commit", "-m", "upstream"], clone)
    git(["push"], clone)
    output = StringIO()
    update(Config.load(sample.config_path()), sample.repo(), output, False, True)
    text = output.getvalue()
    assert "rebased 'main'" in text
    log = git(["log", "--oneline"], sub)
    assert "Merge" not in log


def test_update_merges_by_default(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = sample.subrepo_path("sub-a")
    git(["config", "pull.rebase", "false"], sub)
    git(["add", "."], sample.root)
    git(["commit", "-m", "baseline"], sample.root)
    remote = sample.root / "remotes" / "sub-a.git"
    remote.parent.mkdir()
    git(["init", "--bare", remote.name], remote.parent)
    git(["remote", "add", "origin", str(remote)], sub)
    git(["push", "-u", "origin", "main"], sub)
    (sub / "LOCAL.md").write_text("local")
    git(["add", "LOCAL.md"], sub)
    git(["commit", "-m", "local commit"], sub)
    clone = sample.root / "remotes" / "contributor"
    git(["clone", str(remote), str(clone)], remote.parent)
    git(["config", "user.email", "test@localhost"], clone)
    git(["config", "user.name", "Test User"], clone)
    (clone / "UPSTREAM.md").write_text("upstream")
    git(["add", "UPSTREAM.md"], clone)
    git(["commit", "-m", "upstream"], clone)
    git(["push"], clone)
    output = StringIO()
    update(Config.load(sample.config_path()), sample.repo(), output, False, True)
    assert "merged 'main'" in output.getvalue()


def test_update_no_commit(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    git(["add", "."], sample.root)
    git(["commit", "-m", "baseline"], sample.root)
    sub = sample.subrepo_path("sub-a")
    remote_parent = sample.root.parent / "remotes"
    remote_parent.mkdir()
    remote = remote_parent / "sub-a.git"
    git(["init", "--bare", str(remote)], remote_parent)
    git(["remote", "add", "origin", str(remote)], sub)
    git(["push", "-u", "origin", "main"], sub)
    clone = remote_parent / "contributor"
    git(["clone", str(remote), str(clone)], remote_parent)
    git(["config", "user.email", "test@localhost"], clone)
    git(["config", "user.name", "Test User"], clone)
    git(["commit", "--allow-empty", "-m", "upstream"], clone)
    git(["push"], clone)
    before = git(["rev-parse", "HEAD"], sample.root).strip()
    output = StringIO()
    update(Config.load(sample.config_path()), sample.repo(), output, True, True)
    text = output.getvalue()
    assert "fast-forwarded 'main'" in text
    assert "commit skipped" in text
    assert git(["rev-parse", "HEAD"], sample.root).strip() == before


def test_update_initializes_missing_submodule(repo_sample) -> None:
    from typer.testing import CliRunner

    from wok.cli import app
    from wok.config import Config as WokConfig

    sample = repo_sample([], "empty.toml")
    source = sample.root.parent / "source"
    source.mkdir()
    git(["init", "-b", "main"], source)
    git(["config", "user.email", "test@localhost"], source)
    git(["config", "user.name", "Test User"], source)
    (source / "README.md").write_text("sub-b initial")
    git(["add", "README.md"], source)
    git(["commit", "-m", "initial"], source)
    remote = sample.root.parent / "sub-b.git"
    git(["init", "--bare", str(remote)], sample.root.parent)
    git(["remote", "add", "origin", str(remote)], source)
    git(["push", "-u", "origin", "main"], source)
    git(
        ["-c", "protocol.file.allow=always", "submodule", "add", str(remote), "sub-b"],
        sample.root,
    )
    config = WokConfig.load(sample.config_path())
    config.add_repo(Path("sub-b"), "main")
    config.save(sample.config_path())
    git(["add", ".gitmodules", "sub-b", "wok.toml"], sample.root)
    git(["commit", "-m", "configure sub-b"], sample.root)
    clone = sample.root.parent / "consumer"
    git(["clone", str(sample.root), str(clone)], sample.root.parent)
    git(["config", "user.email", "test@localhost"], clone)
    git(["config", "user.name", "Test User"], clone)
    result = CliRunner().invoke(
        app, ["-f", str(clone / "wok.toml"), "update", "--no-commit"]
    )
    assert result.exit_code == 0, result.output + result.stderr
    assert (clone / "sub-b" / "README.md").read_text() == "sub-b initial"


def test_update_skips_umbrella(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    output = StringIO()
    update(Config.load(sample.config_path()), sample.repo(), output, False, False)
    assert "umbrella" not in output.getvalue()


def test_update_unknown_flag() -> None:
    from typer.testing import CliRunner

    from wok.cli import app

    result = CliRunner().invoke(app, ["update", "--not-a-flag"])
    assert result.exit_code != 0


def test_update_conflict_is_reported(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = sample.subrepo_path("sub-a")
    git(["add", "."], sample.root)
    git(["commit", "-m", "baseline"], sample.root)
    remote = sample.root / "remotes" / "sub-a.git"
    remote.parent.mkdir()
    git(["init", "--bare", remote.name], remote.parent)
    git(["remote", "add", "origin", str(remote)], sub)
    (sub / "SAME.md").write_text("base\n")
    git(["add", "SAME.md"], sub)
    git(["commit", "-m", "base file"], sub)
    git(["push", "-u", "origin", "main"], sub)
    clone = sample.root / "remotes" / "contributor"
    git(["clone", str(remote), str(clone)], remote.parent)
    git(["config", "user.email", "test@localhost"], clone)
    git(["config", "user.name", "Test User"], clone)
    (clone / "SAME.md").write_text("upstream\n")
    git(["add", "SAME.md"], clone)
    git(["commit", "-m", "upstream"], clone)
    git(["push"], clone)
    (sub / "SAME.md").write_text("local\n")
    git(["add", "SAME.md"], sub)
    git(["commit", "-m", "local"], sub)
    output = StringIO()
    update(Config.load(sample.config_path()), sample.repo(), output, False, False)
    assert "conflict" in output.getvalue().lower()
