from pathlib import Path

import pytest
from tests.conftest import expected_config

from wok.cmd.repo import add, rm
from wok.config import Config
from wok.errors import WokError


def test_add_submodule(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    config = Config.load(sample.config_path())
    path = sample.add_submodule("sub-a")
    assert add(config, sample.repo(), path)
    assert config.dump() == expected_config("a.toml")


def test_add_missing_submodule_fails(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    config = Config.load(sample.config_path())
    with pytest.raises(WokError, match="Cannot find submodule"):
        add(config, sample.repo(), Path("missing"))


def test_add_existing_repo_is_a_noop(repo_sample) -> None:
    sample = repo_sample(["sub-a"], "a.toml")
    config = Config.load(sample.config_path())
    assert add(config, sample.repo(), Path("sub-a")) is False


def test_remove_submodule(repo_sample) -> None:
    sample = repo_sample([], "a.toml")
    config = Config.load(sample.config_path())
    path = sample.add_submodule("sub-a")
    assert rm(config, path)
    assert config.dump() == expected_config("empty.toml")


def test_remove_missing_is_a_noop(repo_sample) -> None:
    sample = repo_sample([], "empty.toml")
    config = Config.load(sample.config_path())
    assert rm(config, Path("missing")) is False
