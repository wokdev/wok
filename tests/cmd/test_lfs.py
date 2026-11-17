from io import StringIO
from pathlib import Path

import pytest
from tests.conftest import git, has_git_lfs

from wok.cmd import push
from wok.config import Config
from wok.git import lfs_missing_message


def test_lfs_missing_message_names_fedora() -> None:
    text = lfs_missing_message(Path("repo"))
    assert "sudo dnf install git-lfs" in text
    assert "git lfs install" in text


@pytest.mark.skipif(not has_git_lfs(), reason="git-lfs is not installed")
def test_push_uploads_lfs_objects(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    sub = sample.subrepo_path("sub-a")
    git(["lfs", "install", "--local"], sub)
    (sub / ".gitattributes").write_text("*.bin filter=lfs diff=lfs merge=lfs -text\n")
    (sub / "blob.bin").write_bytes(b"not-a-pointer")
    git(["add", ".gitattributes", "blob.bin"], sub)
    git(["commit", "-m", "lfs"], sub)
    remote = sample.root / "sub-a.git"
    git(["init", "--bare", remote.name], sample.root)
    git(["lfs", "install", "--local"], remote)
    git(["remote", "add", "origin", str(remote)], sub)
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
    assert "pushed to 'main'" in output.getvalue()
    assert "filter=lfs" in (sub / ".gitattributes").read_text()
