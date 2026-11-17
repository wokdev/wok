from io import StringIO
from pathlib import Path

import pytest
from tests.conftest import git, has_git_lfs

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


def test_update_skips_configured_repo(repo_sample) -> None:
    sample = repo_sample(["sub-a", "sub-b"], "a-b-skip.toml")
    output = StringIO()
    update(Config.load(sample.config_path()), sample.repo(), output, False, True)
    text = output.getvalue()
    assert "Updating repositories..." in text
    assert "- 'umbrella':" in text
    assert "- 'sub-a':" not in text
    assert "- 'sub-b':" in text


def test_update_pulls_tracking_branch(repo_sample) -> None:
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
    (clone / "UPSTREAM.md").write_text("upstream change")
    git(["add", "UPSTREAM.md"], clone)
    git(["commit", "-m", "upstream"], clone)
    git(["push"], clone)
    local_before = git(["rev-parse", "HEAD"], sub).strip()
    umbrella_before = git(["rev-parse", "HEAD"], sample.root).strip()
    output = StringIO()
    update(Config.load(sample.config_path()), sample.repo(), output, False, True)
    local_after = git(["rev-parse", "HEAD"], sub).strip()
    remote_tip = git(["rev-parse", "origin/main"], sub).strip()
    assert local_before != local_after
    assert local_after == remote_tip
    assert umbrella_before != git(["rev-parse", "HEAD"], sample.root).strip()
    text = output.getvalue()
    assert "Updating repositories..." in text
    assert "- 'sub-a': fast-forwarded 'main'" in text
    assert "- 'umbrella':" in text
    assert "Updated submodule state committed" in text


def test_update_switches_detached_submodule_to_configured_branch(repo_sample) -> None:
    sample = repo_sample([])
    remote_parent = sample.root / "remotes"
    remote_parent.mkdir()
    source = remote_parent / "source"
    source.mkdir()
    git(["init", "-b", "main"], source)
    git(["config", "user.email", "test@localhost"], source)
    git(["config", "user.name", "Test User"], source)
    (source / "README.md").write_text("initial")
    git(["add", "README.md"], source)
    git(["commit", "-m", "initial"], source)
    remote = remote_parent / "sub-a.git"
    git(["init", "--bare", remote.name], remote_parent)
    git(["remote", "add", "origin", str(remote)], source)
    git(["push", "-u", "origin", "main"], source)
    git(
        ["-c", "protocol.file.allow=always", "submodule", "add", str(remote), "sub-a"],
        sample.root,
    )
    git(
        ["-c", "protocol.file.allow=always", "submodule", "update", "--init"],
        sample.root,
    )
    config = Config.new()
    config.add_repo(Path("sub-a"), "main")
    config.save(sample.config_path())
    sub = sample.root / "sub-a"
    git(["checkout", "--detach"], sub)
    assert git(["rev-parse", "--abbrev-ref", "HEAD"], sub).strip() == "HEAD"
    output = StringIO()
    update(config, sample.repo(), output, False, True)
    assert git(["branch", "--show-current"], sub).strip() == "main"
    upstream = git(
        ["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"], sub
    ).strip()
    assert upstream == "origin/main"


def test_update_pulls_lfs_objects_when_repo_uses_lfs(repo_sample) -> None:
    if not has_git_lfs():
        pytest.skip("git-lfs is not installed")
    sample = repo_sample(["sub-a"], "a.toml")
    sub = sample.subrepo_path("sub-a")
    remote = sample.root / "remotes" / "sub-a.git"
    remote.parent.mkdir()
    git(["init", "--bare", remote.name], remote.parent)
    git(["remote", "add", "origin", str(remote)], sub)
    git(["lfs", "install", "--local"], sub)
    git(["lfs", "track", "*.bin"], sub)
    (sub / "seed.bin").write_text("initial lfs payload")
    git(["add", ".gitattributes", "seed.bin"], sub)
    git(["commit", "-m", "seed lfs"], sub)
    git(["push", "-u", "origin", "main"], sub)
    clone = sample.root / "remotes" / "contributor"
    git(["clone", str(remote), str(clone)], remote.parent)
    git(["config", "user.email", "test@localhost"], clone)
    git(["config", "user.name", "Test User"], clone)
    git(["lfs", "install", "--local"], clone)
    expected = "upstream lfs payload"
    (clone / "UPSTREAM.bin").write_text(expected)
    git(["add", "UPSTREAM.bin"], clone)
    git(["commit", "-m", "upstream-lfs"], clone)
    git(["push"], clone)
    output = StringIO()
    update(Config.load(sample.config_path()), sample.repo(), output, False, True)
    updated = (sub / "UPSTREAM.bin").read_text()
    assert updated == expected
    assert "https://git-lfs.github.com/spec/v1" not in updated


def test_update_with_no_submodules(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    before = git(["rev-parse", "HEAD"], sample.root).strip()
    output = StringIO()
    update(Config.load(sample.config_path()), sample.repo(), output, False, True)
    text = output.getvalue()
    assert "Updating repositories..." in text
    assert "- 'umbrella': already up to date" in text
    assert "No submodule updates detected; nothing to commit" in text
    assert "Updated submodule state committed" not in text
    assert git(["rev-parse", "HEAD"], sample.root).strip() == before


def test_update_without_umbrella_when_no_submodules(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    output = StringIO()
    update(Config.load(sample.config_path()), sample.repo(), output, False, False)
    text = output.getvalue()
    assert "No submodule updates detected; nothing to commit" in text
    assert "- 'umbrella':" not in text
