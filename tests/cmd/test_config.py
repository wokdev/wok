import os
import time
from pathlib import Path

import pytest
from tests.conftest import expected_config

from wok.config import Config
from wok.errors import WokError


def test_dump_matches_fixtures() -> None:
    for name in [
        "empty.toml",
        "a.toml",
        "a-b.toml",
        "a-b-skip.toml",
        "a-b-other.toml",
        "a-other-b.toml",
        "deps-a.toml",
    ]:
        loaded = Config.load(Path("tests/data/configs") / name)
        assert loaded.dump() == expected_config(name)


def test_migrates_experimental_version(repo_sample) -> None:
    sample = repo_sample()
    sample.config_path().write_text('version = "1.0-experimental"\nrepo = []\n')
    config = Config.load(sample.config_path())
    assert config.version == "1.0"
    text = sample.config_path().read_text()
    assert 'version = "1.0"' in text
    assert "experimental" not in text


def test_stable_version_does_not_rewrite(repo_sample) -> None:
    sample = repo_sample()
    sample.config_path().write_text('version = "1.0"\nrepo = []\n')
    os.utime(sample.config_path(), (time.time() - 10, time.time() - 10))
    before = sample.config_path().stat().st_mtime_ns
    config = Config.load(sample.config_path())
    assert config.version == "1.0"
    assert sample.config_path().stat().st_mtime_ns == before


def test_migration_keeps_repos(repo_sample) -> None:
    sample = repo_sample()
    sample.config_path().write_text(
        """version = "1.0-experimental"

[[repo]]
path = "sub-a"
head = "main"

[[repo]]
path = "sub-b"
head = "develop"
skip_for = ["push", "tag"]
"""
    )
    config = Config.load(sample.config_path())
    assert config.version == "1.0"
    assert [repo.path.as_posix() for repo in config.repos] == ["sub-a", "sub-b"]
    assert config.repos[1].head == "develop"
    assert config.repos[1].skip_for == ["push", "tag"]
    text = sample.config_path().read_text()
    assert "experimental" not in text
    assert "develop" in text


def test_unknown_key_is_rejected(tmp_path: Path) -> None:
    path = tmp_path / "wok.toml"
    path.write_text('version = "1.0"\nextra = true\nrepo = []\n')
    with pytest.raises(WokError, match="unknown key"):
        Config.load(path)


def test_skip_for_is_case_insensitive() -> None:
    config = Config.load(Path("tests/data/configs/a-b-skip.toml"))
    assert config.repos[0].is_skipped_for("TAG")
    assert not config.repos[1].is_skipped_for("tag")
